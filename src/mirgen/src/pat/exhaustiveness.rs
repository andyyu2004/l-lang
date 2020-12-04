//! checks match expressions for usefulness and exhaustiveness
//! http://moscova.inria.fr/~maranget/papers/warn/warn.pdf
#![allow(dead_code)]

use super::{MatchCtxt, PatternError};
use crate::LoweringCtx;
use ast::Lit;
use ir::DefId;
use lcore::ty::{tls, Ty, TyKind};
use std::fmt::{self, Display, Formatter};
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
        if !pcx.check_match_exhaustiveness(scrut, arms) {
            self.tcx.sess.emit_error(expr.span, PatternError::NonexhaustiveMatch);
        }
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
        scrut: &ir::Expr<'tcx>,
        arms: &[ir::Arm<'tcx>],
    ) -> bool {
        let matrix: Matrix = arms
            .iter()
            .map(|arm| PatternVector::from_pat(self.lower_ir_pattern(arm.pat)))
            .collect();

        // match is exhaustive if `!is_useful(matrix, wildcard)`
        let wildcard = self.arenaref.alloc(Pat::Wildcard);
        let v = PatternVector::from_pat(wildcard);
        // if wildcard is useful, then it is not exhaustive
        !UsefulnessCtxt { ctx: self, matrix }.is_useful(&v)
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
        match &pat.kind {
            tir::PatternKind::Box(..) | tir::PatternKind::Field(..) =>
                Pat::Ctor(Ctor::Constant, Fields::empty()),
            tir::PatternKind::Binding(..) | tir::PatternKind::Wildcard => Pat::Wildcard,
            tir::PatternKind::Lit(lit) => Pat::Ctor(Ctor::Literal(lit), Fields::empty()),
            tir::PatternKind::Variant(adt, _, idx, pats) => {
                let ctor = Ctor::Variant(adt.variants[*idx].def_id);
                let pats = self
                    .arenaref
                    .alloc_from_iter(pats.iter().map(|pat| self.lower_pattern_inner(pat)));
                let fields = Fields::new(pats);
                Pat::Ctor(ctor, fields)
            }
        }
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
    fn is_useful(&self, v: &PatternVector<'p, 'tcx>) -> bool {
        let Self { matrix, .. } = self;
        println!("matrix:\n{}", matrix);
        println!("vector:\n{}", v);

        assert!(matrix.iter().all(|r| r.len() == v.len()));
        // base case: no columns
        if v.is_empty() {
            // useful if matrix has no rows; useless otherwise
            return matrix.rows.is_empty();
        }

        match v.head_pat() {
            Pat::Ctor(ctor, fields) => {
                let matrix = self.specialize_matrix(ctor, fields);
                let v = self.specialize_vector(v, ctor, fields);
                v.map(|v| Self { matrix, ..*self }.is_useful(&v)).unwrap_or(false)
            }
            Pat::Wildcard =>
                if self.is_complete() {
                    todo!()
                } else {
                    let matrix = self.construct_dmatrix(&self.matrix);
                    println!("dmatrix:\n{}", matrix);
                    let v = PatternVector::new(&v[1..]);
                    Self { matrix, ..*self }.is_useful(&v)
                },
        }
    }

    /// whether the list of head constructors contains all possible constructors
    fn is_complete(&self) -> bool {
        false
    }

    fn all_ctors(&self, ty: Ty<'tcx>) -> Vec<Ctor> {
        match ty.kind {
            TyKind::Adt(adt, _) =>
                adt.variants.iter().map(|variant| Ctor::Variant(variant.def_id)).collect(),
            TyKind::Bool => todo!(),
            _ => unimplemented!(),
        }
    }

    fn construct_dmatrix(&self, matrix: &Matrix<'p, 'tcx>) -> Matrix<'p, 'tcx> {
        let dmatrix = matrix
            .rows
            .iter()
            .filter_map(|row| match row.head_pat() {
                Pat::Ctor(..) => None,
                Pat::Wildcard => Some(PatternVector::new(&row[1..])),
            })
            .collect();
        dmatrix
    }

    /// calculates `S(c, q)`
    fn specialize_vector(
        &self,
        vector: &PatternVector<'p, 'tcx>,
        // ctor of pattern `q`
        qctor: &Ctor,
        // fields of pattern `q`
        qfields: &Fields,
    ) -> Option<PatternVector<'p, 'tcx>> {
        // `row` is `r_1 ... r_a` initially
        let mut row: Vec<Pat> = match vector.head_pat() {
            Pat::Ctor(ctor, fields) => {
                debug_assert_eq!(qfields.len(), fields.len());
                if qctor != ctor {
                    return None;
                }
                fields.pats.to_vec()
            }
            Pat::Wildcard => qfields.into_iter().map(|_| Pat::Wildcard).collect(),
        };
        row.extend_from_slice(&vector[1..]);
        assert_eq!(row.len(), self.matrix.len() + qfields.len() - 1);
        Some(PatternVector::new(self.arenaref.alloc_from_iter(row)))
    }

    /// calculates `S(c, self.matrix)`
    fn specialize_matrix(&self, qctor: &Ctor, qfields: &Fields) -> Matrix<'p, 'tcx> {
        self.matrix
            .rows
            .iter()
            .filter_map(|row| self.specialize_vector(row, qctor, qfields))
            .collect()
    }
}

#[derive(Default, Debug)]
struct Matrix<'p, 'tcx> {
    rows: Vec<PatternVector<'p, 'tcx>>,
}

impl<'p, 'tcx> Matrix<'p, 'tcx> {
    fn head_pats<'a>(&'a self) -> impl Iterator<Item = &'a Pat<'p, 'tcx>> {
        self.rows.iter().map(|r| r.head_pat())
    }
}

impl<'p, 'tcx> Display for Matrix<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "____________________")?;
        for row in &self.rows {
            writeln!(f, "|{}|", row)?;
        }
        writeln!(f, "____________________")
    }
}

impl<'p, 'tcx> FromIterator<PatternVector<'p, 'tcx>> for Matrix<'p, 'tcx> {
    fn from_iter<T: IntoIterator<Item = PatternVector<'p, 'tcx>>>(iter: T) -> Self {
        Self { rows: iter.into_iter().collect() }
    }
}

#[derive(Debug)]
struct PatternVector<'p, 'tcx> {
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

impl<'p, 'tcx> Display for PatternVector<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", util::join2(self.pats, ","))
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

/// pattern as defined in the paper
#[derive(Clone, Debug)]
enum Pat<'p, 'tcx> {
    Ctor(Ctor, Fields<'p, 'tcx>),
    Wildcard,
}

impl<'p, 'tcx> Display for Pat<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Pat::Ctor(ctor, fields) => write!(f, "{}({})", ctor, fields),
            Pat::Wildcard => write!(f, "_"),
        }
    }
}

#[derive(Debug, Clone)]
struct Fields<'p, 'tcx> {
    pats: &'p [Pat<'p, 'tcx>],
    _pd: std::marker::PhantomData<&'tcx ()>,
}

impl<'p, 'tcx> Display for Fields<'p, 'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", util::join2(self.pats, ","))
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Ctor {
    Variant(DefId),
    Literal(Lit),
    /// nullary constructor
    Constant,
}

impl Display for Ctor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ctor::Variant(def_id) =>
                write!(f, "{}", tls::with_tcx(|tcx| tcx.defs().ident(*def_id))),
            Ctor::Literal(lit) => write!(f, "{}", lit),
            Ctor::Constant => write!(f, "unit"),
        }
    }
}
