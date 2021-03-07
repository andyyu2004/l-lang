use crate::LInterner;
use logic_ir::*;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

impl<'tcx> Debug for LInterner<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<interner>")
    }
}

// just copy the impl for IRInterner for now
impl<'tcx> Interner for LInterner<'tcx> {
    type InternedClause = Rc<ClauseData<Self>>;
    type InternedClauses = Vec<Clause<Self>>;
    type InternedGoal = Rc<GoalData<Self>>;
    type InternedGoals = Vec<Goal<Self>>;
    type InternedSubsts = Vec<Term<Self>>;
    type InternedTerm = Rc<TermData<Self>>;
    type InternedTerms = Vec<Term<Self>>;

    fn clause_data<'a>(&self, clause: &'a Self::InternedClause) -> &'a ClauseData<Self> {
        clause
    }

    fn clauses<'a>(&self, clauses: &'a Self::InternedClauses) -> &'a [Clause<Self>] {
        clauses.as_slice()
    }

    fn goal_data<'a>(&self, goal: &'a Self::InternedGoal) -> &'a GoalData<Self> {
        goal
    }

    fn goals<'a>(&self, goals: &'a Self::InternedGoals) -> &'a [Goal<Self>] {
        goals.as_slice()
    }

    fn term_data<'a>(&self, term: &'a Self::InternedTerm) -> &'a TermData<Self> {
        term
    }

    fn terms<'a>(&self, terms: &'a Self::InternedTerms) -> &'a [Term<Self>] {
        terms.as_slice()
    }

    fn intern_goal(self, goal: GoalData<Self>) -> Self::InternedGoal {
        Rc::new(goal)
    }

    fn intern_clause(self, clause: ClauseData<Self>) -> Self::InternedClause {
        Rc::new(clause)
    }

    fn intern_clauses(
        self,
        clauses: impl IntoIterator<Item = Clause<Self>>,
    ) -> Self::InternedClauses {
        clauses.into_iter().collect()
    }

    fn intern_term(self, term: TermData<Self>) -> Self::InternedTerm {
        Rc::new(term)
    }

    fn intern_goals(self, goals: impl IntoIterator<Item = Goal<Self>>) -> Self::InternedGoals {
        goals.into_iter().collect()
    }

    fn intern_terms(self, terms: impl IntoIterator<Item = Term<Self>>) -> Self::InternedTerms {
        terms.into_iter().collect()
    }

    fn intern_substs(self, substs: impl IntoIterator<Item = Term<Self>>) -> Self::InternedSubsts {
        substs.into_iter().collect()
    }
}

// the following traits are required but never used
impl<'tcx> Hash for LInterner<'tcx> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!()
    }
}

impl<'tcx> PartialEq for LInterner<'tcx> {
    fn eq(&self, _other: &Self) -> bool {
        panic!()
    }
}

impl<'tcx> Eq for LInterner<'tcx> {
}
