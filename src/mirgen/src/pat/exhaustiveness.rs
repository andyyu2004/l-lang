//! checks match expressions for usefulness and exhaustiveness
//! http://moscova.inria.fr/~maranget/papers/warn/warn.pdf
#![allow(dead_code)]

use super::{MatchCtxt, PatternError};
use crate::LoweringCtx;
use ir::DefId;
use lcore::ty::{tls, Const, Ty, TyKind};
use std::collections::{HashSet, VecDeque};
use std::fmt::{self, Debug, Display, Formatter};
use std::iter::FromIterator;
use std::ops::Deref;

impl<'a, 'tcx> MatchCtxt<'a, 'tcx> {
    crate fn check_match(
        &self,
        expr: &ir::Expr<'tcx>,
        scrut: &ir::Expr<'tcx>,
        arms: &[ir::Arm<'tcx>],
    ) {
        let pcx = PatCtxt { mcx: self };
        pcx.check_match_exhaustiveness(expr, scrut, arms);
    }
}

/// context for usefulness check
struct PatCtxt<'a, 'tcx> {
    mcx: &'a MatchCtxt<'a, 'tcx>,
}

impl<'a, 'tcx> Deref for PatCtxt<'a, 'tcx> {
    type Target = MatchCtxt<'a, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.mcx
    }
}

impl<'p, 'tcx> PatCtxt<'p, 'tcx> {
    /// returns whether the match is exhaustive
    crate fn check_match_exhaustiveness(
        &self,
        expr: &ir::Expr<'tcx>,
        scrut: &ir::Expr<'tcx>,
        arms: &[ir::Arm<'tcx>],
    ) {
        let matrix: Matrix = arms
            .iter()
            .map(|arm| PatternVector::from_pat(self.lower_ir_pattern(arm.pat)))
            .collect();

        // match is exhaustive if `!is_useful(matrix, wildcard)`
        let ty = self.tables.node_type(scrut.id);
        let wildcard = self.arenaref.alloc(Pat { ty, kind: PatKind::Wildcard });
        let v = PatternVector::from_pat(wildcard);
        let ctxt = UsefulnessCtxt { ctx: self, matrix };
        if let Some(witness) = ctxt.find_uncovered_pattern(&v) {
            self.tcx.sess.emit_error(expr.span, PatternError::NonexhaustiveMatch(witness));
        }
    }

    /// ir pattern to pat
    fn lower_ir_pattern(&self, pat: &ir::Pattern<'tcx>) -> &'p Pat<'p, 'tcx> {
        let mut lctx = LoweringCtx::new(self.tcx, self.tables);
        let tir_pat = lctx.lower_pattern_tir(pat);
        self.lower_pattern(&tir_pat)
    }

    /// tir pattern -> pat
    fn lower_pattern(&self, pat: &tir::Pattern<'tcx>) -> &'p Pat<'p, 'tcx> {
        self.arenaref.alloc(self.lower_pattern_inner(pat))
    }

    fn lower_pattern_inner(&self, pat: &tir::Pattern<'tcx>) -> Pat<'p, 'tcx> {
        let kind = match &pat.kind {
            tir::PatternKind::Box(pat) => {
                let field = self.arenaref.alloc(self.lower_pattern_inner(pat));
                let fields = Fields::new(std::slice::from_ref(field));
                PatKind::Ctor(Ctor::Box, fields)
            }
            tir::PatternKind::Field(fields) => {
                let fields = self
                    .arenaref
                    .alloc_from_iter(fields.iter().map(|f| self.lower_pattern_inner(&f.pat)));
                let ctor = match pat.ty.kind {
                    TyKind::Tuple(..) => Ctor::Tuple,
                    TyKind::Adt(..) => Ctor::Struct,
                    _ => unreachable!(),
                };
                PatKind::Ctor(ctor, Fields::new(fields))
            }
            tir::PatternKind::Binding(..) | tir::PatternKind::Wildcard => PatKind::Wildcard,
            tir::PatternKind::Lit(c) => PatKind::Ctor(Ctor::Literal(c), Fields::empty()),
            tir::PatternKind::Variant(adt, _, idx, pats) => {
                let ctor = Ctor::Variant(adt.variants[*idx].def_id);
                let pats = self
                    .arenaref
                    .alloc_from_iter(pats.iter().map(|pat| self.lower_pattern_inner(pat)));
                let fields = Fields::new(pats);
                PatKind::Ctor(ctor, fields)
            }
        };
        Pat { ty: pat.ty, kind }
    }
}

#[derive(Default, Debug)]
crate struct Witness<'p, 'tcx> {
    pats: Vec<Pat<'p, 'tcx>>,
}

impl<'p, 'tcx> Witness<'p, 'tcx> {
    fn prepend(mut self, pat: Pat<'p, 'tcx>) -> Self {
        self.pats.insert(0, pat);
        self
    }
}

impl<'p, 'tcx> Display for Witness<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", util::join(&self.pats, ","))
    }
}

struct UsefulnessCtxt<'a, 'p, 'tcx> {
    ctx: &'a PatCtxt<'p, 'tcx>,
    matrix: Matrix<'p, 'tcx>,
}

impl<'p, 'tcx> Deref for UsefulnessCtxt<'_, 'p, 'tcx> {
    type Target = PatCtxt<'p, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'a, 'p, 'tcx> UsefulnessCtxt<'a, 'p, 'tcx> {
    fn find_uncovered_pattern(&self, v: &PatternVector<'p, 'tcx>) -> Option<Witness<'p, 'tcx>> {
        let Self { matrix, .. } = self;
        debug!("matrix:\n{:?}", matrix);
        debug!("vector:\n{:?}\n\n", v);

        // base case: no columns
        if v.is_empty() {
            // useful if matrix has no rows; useless otherwise
            return if matrix.rows.is_empty() { Some(Witness::default()) } else { None };
        }

        debug_assert_eq!(matrix.width(), v.len());

        // algorithm `I` (page 18)
        let pat = v.head_pat();
        let ctors = self.matrix.head_ctors().map(|(c, _)| c).copied().collect::<HashSet<_>>();

        if self.ctors_are_complete(&ctors, pat.ty) {
            for (ctor, fields) in self.matrix.head_ctors() {
                if let Some(witness) = self.find_uncovered_ctor(pat, ctor, fields, v) {
                    return Some(witness);
                }
            }
            None
        } else {
            let matrix = self.construct_dmatrix(&self.matrix);
            if !matrix.is_empty() {
                debug_assert_eq!(matrix.width(), self.matrix.width() - 1);
            }
            let q = PatternVector::new(&v[1..]);
            let witness = Self { matrix, ..*self }.find_uncovered_pattern(&q)?;
            debug_assert_eq!(witness.pats.len(), q.len());
            if let Some((&ctor, fields)) = self.matrix.head_ctors().next() {
                // remove an arbitrary constructor as a witness
                // Some(self.apply_ctor(pat, ctor, fields.len(), witness))
                let wildcards = self.arenaref.alloc_from_iter(
                    fields.iter().map(|f| Pat { ty: f.ty, kind: PatKind::Wildcard }),
                );
                let pat = Pat { ty: pat.ty, kind: PatKind::Ctor(ctor, Fields::new(wildcards)) };
                Some(witness.prepend(pat))
            } else {
                let wildcard = Pat { ty: pat.ty, kind: PatKind::Wildcard };
                Some(witness.prepend(wildcard))
            }
        }
    }

    fn apply_ctor(
        &self,
        pat: &Pat<'p, 'tcx>,
        ctor: Ctor<'tcx>,
        arity: usize,
        witness: Witness<'p, 'tcx>,
    ) -> Witness<'p, 'tcx> {
        let args = self.arenaref.alloc_from_iter(witness.pats[..arity].iter().copied());
        let applied = Pat { kind: PatKind::Ctor(ctor, Fields::new(args)), ty: pat.ty };
        let mut pats = vec![applied];
        pats.extend(witness.pats[arity..].iter().copied());
        let new_witness = Witness { pats };
        debug_assert_eq!(new_witness.pats.len() + arity - 1, witness.pats.len());
        new_witness
    }

    fn find_uncovered_ctor(
        &self,
        pat: &Pat<'p, 'tcx>,
        ctor: &Ctor<'tcx>,
        fields: &Fields<'p, 'tcx>,
        v: &PatternVector<'p, 'tcx>,
    ) -> Option<Witness<'p, 'tcx>> {
        debug_assert_eq!(pat.ty, v[0].ty);
        let matrix = self.specialize_matrix(ctor, fields);
        let v = self.specialize_vector(ctor, fields, v)?;
        let witness = Self { matrix, ..*self }.find_uncovered_pattern(&v)?;
        let arity = fields.len();
        Some(self.apply_ctor(pat, *ctor, arity, witness))
    }

    /// whether `ctors` contains all possible constructors wrt `ty`
    fn ctors_are_complete(&self, ctors: &HashSet<Ctor<'tcx>>, ty: Ty<'tcx>) -> bool {
        let all_ctors = self.all_ctors_of_ty(ty);
        if all_ctors.contains(&Ctor::NonExhaustive) {
            return false;
        }
        debug!("{:?} == {:?} = {}", ctors, all_ctors, &all_ctors == ctors);
        ctors == &all_ctors
    }

    /// returns a set of all constructors of `ty`
    fn all_ctors_of_ty(&self, ty: Ty<'tcx>) -> HashSet<Ctor<'tcx>> {
        match ty.kind {
            TyKind::Box(..) => hashset! { Ctor::Box },
            TyKind::Tuple(..) => hashset! { Ctor::Tuple },
            TyKind::Adt(adt, _) if adt.is_enum() =>
                adt.variants.iter().map(|variant| Ctor::Variant(variant.def_id)).collect(),
            TyKind::Adt(..) => hashset! { Ctor::Struct },
            TyKind::Bool => hashset! {
                Ctor::Literal(self.mk_const_bool(true)),
                Ctor::Literal(self.mk_const_bool(false)),
            },
            TyKind::Int => hashset! { Ctor::NonExhaustive },
            _ => unimplemented!("`{}`", ty),
        }
    }

    fn construct_dmatrix(&self, matrix: &Matrix<'p, 'tcx>) -> Matrix<'p, 'tcx> {
        let dmatrix = matrix
            .rows
            .iter()
            .filter_map(|row| match row.head_pat().kind {
                PatKind::Ctor(..) => None,
                PatKind::Wildcard => Some(PatternVector::new(&row[1..])),
            })
            .collect();
        dmatrix
    }

    /// calculates `S(c, q)`
    fn specialize_vector(
        &self,
        // ctor of pattern `q`
        qctor: &Ctor<'tcx>,
        // fields of pattern `q`
        qfields: &Fields<'p, 'tcx>,
        vector: &PatternVector<'p, 'tcx>,
    ) -> Option<PatternVector<'p, 'tcx>> {
        debug!("specialize_vector: {:?} {:?} {:?}", qctor, qfields, vector);
        // `row` is `r_1 ... r_a` initially
        let mut row: Vec<Pat> = match &vector.head_pat().kind {
            PatKind::Ctor(ctor, fields) => {
                if qctor != ctor {
                    return None;
                }
                debug_assert_eq!(qfields.len(), fields.len());
                fields.pats.to_vec()
            }
            PatKind::Wildcard =>
                qfields.into_iter().map(|pat| Pat { ty: pat.ty, kind: PatKind::Wildcard }).collect(),
        };
        row.extend_from_slice(&vector[1..]);
        debug_assert_eq!(row.len(), vector.len() + qfields.len() - 1);
        Some(PatternVector::new(self.arenaref.alloc_from_iter(row)))
    }

    /// calculates `S(c, self.matrix)`
    fn specialize_matrix(
        &self,
        qctor: &Ctor<'tcx>,
        qfields: &Fields<'p, 'tcx>,
    ) -> Matrix<'p, 'tcx> {
        self.matrix
            .rows
            .iter()
            .filter_map(|row| self.specialize_vector(qctor, qfields, row))
            .collect()
    }
}

#[derive(Default)]
struct Matrix<'p, 'tcx> {
    rows: Vec<PatternVector<'p, 'tcx>>,
}

impl<'p, 'tcx> Matrix<'p, 'tcx> {
    // don't call on empty matrix
    fn width(&self) -> usize {
        let width = self.rows[0].len();
        debug_assert!(self.iter().all(|r| r.len() == width));
        width
    }

    fn head_ctors<'a>(&'a self) -> impl Iterator<Item = (&'a Ctor<'tcx>, &'a Fields<'p, 'tcx>)> {
        self.head_pats().filter_map(|pat| pat.ctor())
    }

    fn head_pats<'a>(&'a self) -> impl Iterator<Item = &'a Pat<'p, 'tcx>> {
        self.rows.iter().map(|r| r.head_pat())
    }
}

impl<'p, 'tcx> Debug for Matrix<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "____________________")?;
        for row in &self.rows {
            writeln!(f, "|{:?}|", row)?;
        }
        writeln!(f, "____________________")
    }
}

impl<'p, 'tcx> FromIterator<PatternVector<'p, 'tcx>> for Matrix<'p, 'tcx> {
    fn from_iter<T: IntoIterator<Item = PatternVector<'p, 'tcx>>>(iter: T) -> Self {
        Self { rows: iter.into_iter().collect() }
    }
}

crate struct PatternVector<'p, 'tcx> {
    /// the elements of the (row) vector
    pats: &'p [Pat<'p, 'tcx>],
}

impl<'p, 'tcx> PatternVector<'p, 'tcx> {
    fn new(pats: &'p [Pat<'p, 'tcx>]) -> Self {
        Self { pats }
    }

    fn from_pat(pat: &'p Pat<'p, 'tcx>) -> Self {
        Self::new(std::slice::from_ref(pat))
    }

    fn head_pat(&self) -> &'p Pat<'p, 'tcx> {
        &self.pats[0]
    }
}

impl<'p, 'tcx> Debug for PatternVector<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.pats)
    }
}

impl<'p, 'tcx> Fields<'p, 'tcx> {
    pub fn new(pats: &'p [Pat<'p, 'tcx>]) -> Self {
        Self { pats, _pd: std::marker::PhantomData }
    }

    fn empty() -> Self {
        Self::new(&[])
    }
}

impl<'p, 'tcx> Deref for Fields<'p, 'tcx> {
    type Target = &'p [Pat<'p, 'tcx>];

    fn deref(&self) -> &Self::Target {
        &self.pats
    }
}

#[derive(Clone, Copy)]
crate struct Pat<'p, 'tcx> {
    ty: Ty<'tcx>,
    kind: PatKind<'p, 'tcx>,
}

/// pattern as defined in the paper
#[derive(Clone, Copy)]
enum PatKind<'p, 'tcx> {
    Ctor(Ctor<'tcx>, Fields<'p, 'tcx>),
    Wildcard,
}

impl<'p, 'tcx> Display for Pat<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl<'p, 'tcx> Debug for Pat<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{}", self.kind, self.ty)
    }
}

impl<'p, 'tcx> Pat<'p, 'tcx> {
    fn ctor(&self) -> Option<(&Ctor<'tcx>, &Fields<'p, 'tcx>)> {
        match &self.kind {
            PatKind::Ctor(ctor, fields) => Some((ctor, fields)),
            PatKind::Wildcard => None,
        }
    }
}

impl<'p, 'tcx> Debug for PatKind<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PatKind::Ctor(ctor, fields) => write!(f, "{:?}({:?})", ctor, fields),
            PatKind::Wildcard => write!(f, "_"),
        }
    }
}

impl<'p, 'tcx> Display for PatKind<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PatKind::Ctor(ctor, fields) => match ctor {
                Ctor::Box => write!(f, "&{}", fields),
                Ctor::Variant(def_id) =>
                    write!(f, "{}({})", tls::with_tcx(|tcx| tcx.defs().ident(*def_id)), fields),
                Ctor::Literal(c) => write!(f, "{}", c),
                Ctor::Tuple => write!(f, "({})", fields),
                Ctor::NonExhaustive | Ctor::Struct => todo!(),
            },
            PatKind::Wildcard => write!(f, "_"),
        }
    }
}

#[derive(Clone, Copy)]
struct Fields<'p, 'tcx> {
    pats: &'p [Pat<'p, 'tcx>],
    _pd: std::marker::PhantomData<&'tcx ()>,
}

impl<'p, 'tcx> Display for Fields<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", util::join2(self.pats, ", "))
    }
}

impl<'p, 'tcx> Debug for Fields<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.pats)
    }
}

impl<'p, 'tcx> Deref for Matrix<'p, 'tcx> {
    type Target = Vec<PatternVector<'p, 'tcx>>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl<'p, 'tcx> Deref for PatternVector<'p, 'tcx> {
    type Target = &'p [Pat<'p, 'tcx>];

    fn deref(&self) -> &Self::Target {
        &self.pats
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Ctor<'tcx> {
    Box,
    Variant(DefId),
    Literal(&'tcx Const<'tcx>),
    Tuple,
    NonExhaustive,
    Struct,
}

impl Debug for Ctor<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ctor::Box => write!(f, "box"),
            Ctor::Variant(def_id) =>
                write!(f, "{}", tls::with_tcx(|tcx| tcx.defs().ident(*def_id))),
            Ctor::Literal(lit) => write!(f, "{}", lit),
            Ctor::NonExhaustive => write!(f, "nonexhaustive"),
            Ctor::Tuple => write!(f, "tuple"),
            Ctor::Struct => write!(f, "struct"),
        }
    }
}
