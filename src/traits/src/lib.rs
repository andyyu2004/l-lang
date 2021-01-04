use lcore::ty::{Ty, TyCtx};
use logic_ir::Interner;

#[derive(Copy, Clone, Debug, Hash)]
pub struct LInterner<'tcx> {
    tcx: TyCtx<'tcx>,
}

fn ty_to_term<'tcx>(tcx: TyCtx<'tcx>, ty: Ty<'tcx>) -> logic_ir::Term<LInterner<'tcx>> {
}

impl<'tcx> Interner for LInterner<'tcx> {
    type InternedClause = ();
    type InternedClauses = ();
    type InternedGoal = ();
    type InternedGoals = ();
    type InternedSubsts = ();
    type InternedTerm = ();
    type InternedTerms = ();

    fn clause_data<'a>(&self, clause: &'a Self::InternedClause) -> &'a logic_ir::ClauseData<Self> {
        todo!()
    }

    fn clauses<'a>(&self, clauses: &'a Self::InternedClauses) -> &'a [logic_ir::Clause<Self>] {
        todo!()
    }

    fn goal_data<'a>(&self, goal: &'a Self::InternedGoal) -> &'a logic_ir::GoalData<Self> {
        todo!()
    }

    fn goals<'a>(&self, goals: &'a Self::InternedGoals) -> &'a [logic_ir::Goal<Self>] {
        todo!()
    }

    fn term_data<'a>(&self, term: &'a Self::InternedTerm) -> &'a logic_ir::TermData<Self> {
        todo!()
    }

    fn terms<'a>(&self, terms: &'a Self::InternedTerms) -> &'a [logic_ir::Term<Self>] {
        todo!()
    }

    fn intern_goal(self, goal: logic_ir::GoalData<Self>) -> Self::InternedGoal {
        todo!()
    }

    fn intern_clause(self, clause: logic_ir::ClauseData<Self>) -> Self::InternedClause {
        todo!()
    }

    fn intern_term(self, term: logic_ir::TermData<Self>) -> Self::InternedTerm {
        todo!()
    }

    fn intern_substs(
        self,
        subst: impl IntoIterator<Item = logic_ir::Term<Self>>,
    ) -> Self::InternedSubsts {
        todo!()
    }

    fn intern_goals(
        self,
        goals: impl IntoIterator<Item = logic_ir::Goal<Self>>,
    ) -> Self::InternedGoals {
        todo!()
    }

    fn intern_clauses(
        self,
        clauses: impl IntoIterator<Item = logic_ir::Clause<Self>>,
    ) -> Self::InternedClauses {
        todo!()
    }

    fn intern_terms(
        self,
        terms: impl IntoIterator<Item = logic_ir::Term<Self>>,
    ) -> Self::InternedTerms {
        todo!()
    }
}
