//! checks match expressions for usefulness and exhaustiveness
//! http://moscova.inria.fr/~maranget/papers/warn/warn.pdf
#![allow(dead_code)]

use super::MatchCtxt;
use ir::DefId;
use smallvec::SmallVec;
use std::iter::FromIterator;
use std::ops::Deref;

impl<'a, 'tcx> MatchCtxt<'a, 'tcx> {
    crate fn check_match(&self, scrut: &ir::Expr<'tcx>, arms: &[ir::Arm<'tcx>]) {
        let pcx = PatCtxt { mcx: self };
        pcx.check_match_exhaustiveness(scrut, arms)
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
    crate fn check_match_exhaustiveness(&self, scrut: &ir::Expr<'tcx>, arms: &[ir::Arm<'tcx>]) {
        // let matrix = arms.iter().map(|arm| self.lower_pattern(arm.pat)).collect();
    }

    fn lower_pattern(&self, pat: &tir::Pattern<'tcx>) -> &'p Pat<'p, 'tcx> {
        let pat = self.lower_pattern_inner(pat);
        self.arena.alloc(pat)
    }

    fn lower_pattern_inner(&self, pat: &tir::Pattern<'tcx>) -> Pat<'p, 'tcx> {
        match &pat.kind {
            tir::PatternKind::Box(..) | tir::PatternKind::Field(..) =>
                Pat::Ctor(Ctor::Unit, Fields::empty()),
            tir::PatternKind::Binding(..) | tir::PatternKind::Wildcard => Pat::Wildcard,
            tir::PatternKind::Lit(..) => todo!(),
            tir::PatternKind::Variant(adt, _, idx, pats) => {
                let ctor = Ctor::Variant(adt.variants[*idx].def_id);
                let pats = pats.iter().map(|pat| self.lower_pattern(pat)).collect();
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
        assert!(matrix.iter().all(|r| r.len() == v.len()));
        // base case: no columns
        if v.is_empty() {
            // useful if matrix has no rows; useless otherwise
            return matrix.is_empty();
        }

        match v.head_pat() {
            Pat::Ctor(ctor, fields) => {
                let matrix = self.specialize_matrix(ctor, fields);
                let v = self.specialize_vector(v, ctor, fields);
                v.map(|v| Self { ctx: self.ctx, matrix }.is_useful(&v)).unwrap_or(false)
            }
            Pat::Wildcard => todo!(),
        }
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
        // `row` is `r_1 ... r_a`
        let mut row = match vector.head_pat() {
            Pat::Ctor(ctor, fields) => {
                if qctor != ctor {
                    return None;
                }
                fields.pats.clone()
            }
            Pat::Wildcard => qfields.iter().map(|_| &*self.arena.alloc(Pat::Wildcard)).collect(),
        };
        row.extend_from_slice(&vector[1..]);
        assert_eq!(row.len(), self.matrix.len() + qfields.len() - 1);
        Some(PatternVector::new(row))
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

impl<'p, 'tcx> FromIterator<PatternVector<'p, 'tcx>> for Matrix<'p, 'tcx> {
    fn from_iter<T: IntoIterator<Item = PatternVector<'p, 'tcx>>>(iter: T) -> Self {
        Self { rows: iter.into_iter().collect() }
    }
}

#[derive(Debug)]
struct PatternVector<'p, 'tcx> {
    /// the elements of the (row) vector
    pats: SmallVec<[&'p Pat<'p, 'tcx>; 2]>,
}

#[derive(Debug, Clone)]
struct Fields<'p, 'tcx> {
    pats: SmallVec<[&'p Pat<'p, 'tcx>; 2]>,
    _pd: std::marker::PhantomData<&'tcx ()>,
}

impl<'p, 'tcx> Fields<'p, 'tcx> {
    pub fn new(pats: SmallVec<[&'p Pat<'p, 'tcx>; 2]>) -> Self {
        Self { pats, _pd: std::marker::PhantomData }
    }

    fn empty() -> Self {
        Self::new(smallvec![])
    }
}

impl<'p, 'tcx> Deref for Fields<'p, 'tcx> {
    type Target = [&'p Pat<'p, 'tcx>];

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

impl<'p, 'tcx> PatternVector<'p, 'tcx> {
    fn new(pats: SmallVec<[&'p Pat<'p, 'tcx>; 2]>) -> Self {
        Self { pats }
    }

    fn from_pat(pat: &'p Pat<'p, 'tcx>) -> Self {
        Self::new(smallvec![pat])
    }

    fn head_pat(&self) -> &'p Pat<'p, 'tcx> {
        &self.pats[0]
    }
}

impl<'p, 'tcx> Deref for Matrix<'p, 'tcx> {
    type Target = Vec<PatternVector<'p, 'tcx>>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl<'p, 'tcx> Deref for PatternVector<'p, 'tcx> {
    type Target = SmallVec<[&'p Pat<'p, 'tcx>; 2]>;

    fn deref(&self) -> &Self::Target {
        &self.pats
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Ctor {
    Variant(DefId),
    Unit,
}
