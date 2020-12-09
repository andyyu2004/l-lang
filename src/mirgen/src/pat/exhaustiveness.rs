//! checks match expressions for usefulness and exhaustiveness
//! "http://moscova.inria.fr/~maranget/papers/warn/warn.pdf"
#![allow(dead_code)]

use super::{MatchCtxt, PatternError};
use indexmap::{indexset, IndexSet};
use ir::DefId;
use lcore::ty::{tls, Const, Substs, SubstsRef, Ty, TyKind};
use span::Span;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

impl<'p, 'tcx> MatchCtxt<'p, 'tcx> {
    crate fn check_match(&self, span: Span, scrut: &tir::Expr<'tcx>, arms: &[tir::Arm<'tcx>]) {
        self.check_match_exhaustiveness(span, scrut, arms);
    }

    /// returns whether the match is exhaustive
    crate fn check_match_exhaustiveness(
        &self,
        span: Span,
        scrut: &tir::Expr<'tcx>,
        arms: &[tir::Arm<'tcx>],
    ) {
        let matrix = Matrix::default();
        let mut ucx = UsefulnessCtxt { pcx: self, matrix };
        for arm in arms {
            // check usefulness of each arm
            let v = PatternVector::from_pat(self.lower_pattern(&arm.pat));
            if ucx.find_uncovered_pattern(&v).is_none() {
                self.tcx.sess.emit_warning(arm.span, PatternError::RedundantPattern);
            }
            ucx.matrix.push(v);
        }

        // match is exhaustive iff `!is_useful(matrix, wildcard)`
        let wildcard = self.arena.alloc(Pat::new(scrut.ty, PatKind::Wildcard));
        let v = PatternVector::from_pat(wildcard);
        if let Some(witness) = ucx.find_uncovered_pattern(&v) {
            self.tcx.sess.emit_error(span, PatternError::NonexhaustiveMatch(witness));
        }
    }

    /// tir pattern -> pat
    fn lower_pattern(&self, pat: &tir::Pattern<'tcx>) -> &'p Pat<'p, 'tcx> {
        self.lcx.arena.alloc(self.lower_pattern_inner(pat))
    }

    fn lower_pattern_inner(&self, pat: &tir::Pattern<'tcx>) -> Pat<'p, 'tcx> {
        let kind = match &pat.kind {
            tir::PatternKind::Box(pat) => {
                let field = self.lcx.arena.alloc(self.lower_pattern_inner(pat));
                let fields = Fields::new(std::slice::from_ref(field));
                let field_tys = self.intern_substs(&[pat.ty]);
                let ctor = Ctor::new(field_tys, CtorKind::Box);
                PatKind::Ctor(ctor, fields)
            }
            tir::PatternKind::Field(fields) => {
                let fields = self
                    .lcx
                    .arena
                    .alloc_from_iter(fields.iter().map(|f| self.lower_pattern_inner(&f.pat)));
                let ctor_kind = match pat.ty.kind {
                    TyKind::Tuple(..) => CtorKind::Tuple,
                    TyKind::Adt(..) => CtorKind::Struct,
                    _ => unreachable!(),
                };
                let field_tys = self.mk_substs(fields.iter().map(|f| f.ty));
                PatKind::Ctor(Ctor::new(field_tys, ctor_kind), Fields::new(fields))
            }
            tir::PatternKind::Binding(..) | tir::PatternKind::Wildcard => PatKind::Wildcard,
            tir::PatternKind::Lit(c) => {
                let ctor = Ctor::nullary(CtorKind::Literal(c));
                PatKind::Ctor(ctor, Fields::empty())
            }
            tir::PatternKind::Variant(adt, _, idx, pats) => {
                let variant = &adt.variants[*idx];
                let ctor_kind = CtorKind::Variant(variant.def_id);
                let field_tys = self.mk_substs(pats.iter().map(|pat| pat.ty));
                let ctor = Ctor::new(field_tys, ctor_kind);
                let pats = self
                    .lcx
                    .arena
                    .alloc_from_iter(pats.iter().map(|pat| self.lower_pattern_inner(pat)));
                let fields = Fields::new(pats);
                PatKind::Ctor(ctor, fields)
            }
        };
        Pat::new(pat.ty, kind)
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
        // should this always be true?
        assert_eq!(self.pats.len(), 1);
        write!(f, "{}", util::join(&self.pats, ","))
    }
}

struct UsefulnessCtxt<'a, 'p, 'tcx> {
    pcx: &'a MatchCtxt<'p, 'tcx>,
    matrix: Matrix<'p, 'tcx>,
}

impl<'p, 'tcx> Deref for UsefulnessCtxt<'_, 'p, 'tcx> {
    type Target = MatchCtxt<'p, 'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.pcx
    }
}

impl<'a, 'p, 'tcx> UsefulnessCtxt<'a, 'p, 'tcx> {
    fn find_uncovered_pattern(&self, v: &PatternVector<'p, 'tcx>) -> Option<Witness<'p, 'tcx>> {
        let Self { matrix, .. } = self;

        // base case: no columns
        if v.is_empty() {
            // useful if matrix has no rows; useless otherwise
            return if matrix.rows.is_empty() { Some(Witness::default()) } else { None };
        }

        if !matrix.rows.is_empty() {
            debug_assert_eq!(matrix.width(), v.len());
        }

        // algorithm `I` (page 18)
        let pat = v.head_pat();
        let ctors = self.matrix.head_ctors().map(|(c, _)| c).copied().collect::<IndexSet<_>>();

        if self.ctors_are_complete(&ctors, pat.ty) {
            for (ctor, fields) in self.matrix.head_ctors() {
                if let Some(witness) = self.find_uncovered_ctor(pat, ctor, fields, v) {
                    return Some(witness);
                }
            }
            None
        } else {
            let matrix = self.construct_dmatrix(&self.matrix);
            let q = PatternVector::new(&v[1..]);
            let witness = Self { matrix, ..*self }.find_uncovered_pattern(&q)?;
            debug_assert_eq!(witness.pats.len(), q.len());
            let witness = if ctors.is_empty() {
                let wildcard = Pat { ty: pat.ty, kind: PatKind::Wildcard };
                witness.prepend(wildcard)
            } else {
                // know this witness exists as it is nonexhaustive
                // find an arbitrary constructor as an example witness
                let ctor_witness = self.find_missing_ctor(&ctors, pat.ty).unwrap();
                let wildcards = self.lcx.arena.alloc_from_iter(
                    ctor_witness.field_tys.iter().map(|ty| Pat { ty, kind: PatKind::Wildcard }),
                );
                let pat =
                    Pat { ty: pat.ty, kind: PatKind::Ctor(ctor_witness, Fields::new(wildcards)) };
                witness.prepend(pat)
            };
            debug_assert_eq!(witness.pats.len(), v.len());
            Some(witness)
        }
    }

    fn apply_ctor(
        &self,
        pat: &Pat<'p, 'tcx>,
        ctor: Ctor<'tcx>,
        witness: Witness<'p, 'tcx>,
    ) -> Witness<'p, 'tcx> {
        let arity = ctor.arity();
        let args = self.lcx.arena.alloc_from_iter(witness.pats[..arity].iter().copied());
        let applied = Pat { kind: PatKind::Ctor(ctor, Fields::new(args)), ty: pat.ty };
        let mut pats = vec![applied];
        pats.extend(witness.pats[arity..].iter().copied());
        debug_assert_eq!(pats.len() + arity - 1, witness.pats.len());
        Witness { pats }
    }

    fn find_uncovered_ctor(
        &self,
        pat: &Pat<'p, 'tcx>,
        ctor: &Ctor<'tcx>,
        fields: &Fields<'p, 'tcx>,
        v: &PatternVector<'p, 'tcx>,
    ) -> Option<Witness<'p, 'tcx>> {
        debug_assert_eq!(pat.ty, v[0].ty);
        debug_assert_eq!(ctor.arity(), fields.len());
        let matrix = self.specialize_matrix(ctor, fields);
        let v = self.specialize_vector(ctor, fields, v)?;
        let witness = Self { matrix, ..*self }.find_uncovered_pattern(&v)?;
        Some(self.apply_ctor(pat, *ctor, witness))
    }

    fn find_missing_ctor(&self, ctors: &IndexSet<Ctor<'tcx>>, ty: Ty<'tcx>) -> Option<Ctor<'tcx>> {
        let all_ctors = self.all_ctors_of_ty(ty);
        debug!("{:?} == {:?} = {}", ctors, all_ctors, &all_ctors == ctors);
        all_ctors.difference(ctors).collect::<IndexSet<_>>().pop().copied()
    }

    /// whether `ctors` contains all possible constructors wrt `ty`
    fn ctors_are_complete(&self, ctors: &IndexSet<Ctor<'tcx>>, ty: Ty<'tcx>) -> bool {
        self.find_missing_ctor(ctors, ty).is_none()
    }

    /// returns a set of all constructors of `ty`
    fn all_ctors_of_ty(&self, ty: Ty<'tcx>) -> IndexSet<Ctor<'tcx>> {
        match ty.kind {
            TyKind::Box(ty) => indexset! { Ctor::new(self.intern_substs(&[ty]), CtorKind::Box) },
            TyKind::Tuple(tys) => indexset! { Ctor::new(tys, CtorKind::Tuple) },
            TyKind::Adt(adt, _) => adt
                .variants
                .iter()
                .map(|variant| {
                    let field_tys =
                        self.mk_substs(variant.fields.iter().map(|f| self.type_of(f.def_id)));
                    let kind = CtorKind::Variant(variant.def_id);
                    Ctor::new(field_tys, kind)
                })
                .collect(),
            TyKind::Bool => indexset! {
                Ctor::nullary(CtorKind::Literal(self.mk_const_bool(true))),
                Ctor::nullary(CtorKind::Literal(self.mk_const_bool(false))),
            },
            TyKind::Int => indexset! { Ctor::nullary(CtorKind::NonExhaustive) },
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
            .collect::<Matrix>();
        if !dmatrix.is_empty() {
            debug_assert_eq!(dmatrix.width(), matrix.width() - 1);
        }
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
        Some(PatternVector::new(self.lcx.arena.alloc_from_iter(row)))
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
    // don't call on empty matrix (will panic)
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

impl<'p, 'tcx> Pat<'p, 'tcx> {
    fn new(ty: Ty<'tcx>, kind: PatKind<'p, 'tcx>) -> Self {
        match kind {
            PatKind::Ctor(ctor, fields) => {
                debug_assert_eq!(ctor.field_tys.len(), fields.len());
            }
            PatKind::Wildcard => {}
        }
        Self { ty, kind }
    }
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
            PatKind::Ctor(ctor, fields) => match ctor.kind {
                CtorKind::Box => write!(f, "&{}", fields),
                CtorKind::Variant(def_id) =>
                    write!(f, "{}({})", tls::with_tcx(|tcx| tcx.defs().ident(def_id)), fields),
                CtorKind::Literal(c) => write!(f, "{}", c),
                CtorKind::Tuple => write!(f, "({})", fields),
                CtorKind::NonExhaustive | CtorKind::Struct => todo!(),
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

impl<'p, 'tcx> DerefMut for Matrix<'p, 'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rows
    }
}

impl<'p, 'tcx> Deref for PatternVector<'p, 'tcx> {
    type Target = &'p [Pat<'p, 'tcx>];

    fn deref(&self) -> &Self::Target {
        &self.pats
    }
}

#[derive(Copy, Clone, Eq)]
struct Ctor<'tcx> {
    field_tys: SubstsRef<'tcx>,
    kind: CtorKind<'tcx>,
}

/// important to ignore the types in comparison and hashing
impl<'tcx> PartialEq for Ctor<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'tcx> Hash for Ctor<'tcx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state)
    }
}

impl<'tcx> Ctor<'tcx> {
    fn new(field_tys: SubstsRef<'tcx>, kind: CtorKind<'tcx>) -> Self {
        Self { field_tys, kind }
    }

    fn nullary(kind: CtorKind<'tcx>) -> Self {
        Self::new(Substs::empty(), kind)
    }

    fn arity(&self) -> usize {
        self.field_tys.len()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum CtorKind<'tcx> {
    Box,
    Variant(DefId),
    Literal(&'tcx Const<'tcx>),
    Tuple,
    NonExhaustive,
    Struct,
}

impl Debug for Ctor<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self.kind, self.field_tys)
    }
}

impl Debug for CtorKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CtorKind::Box => write!(f, "box"),
            CtorKind::Variant(def_id) =>
                write!(f, "{}", tls::with_tcx(|tcx| tcx.defs().ident(*def_id))),
            CtorKind::Literal(lit) => write!(f, "{}", lit),
            CtorKind::NonExhaustive => write!(f, "nonexhaustive"),
            CtorKind::Tuple => write!(f, "tuple"),
            CtorKind::Struct => write!(f, "struct"),
        }
    }
}
