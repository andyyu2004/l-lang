mod build;
mod exhaustiveness;

use crate::LoweringCtx;
use exhaustiveness::Witness;
use std::ops::Deref;
use thiserror::Error;

pub(crate) struct MatchCtxt<'p, 'tcx> {
    pub(crate) lcx: &'p LoweringCtx<'tcx>,
}

#[derive(Debug, Error)]
enum PatternError<'p, 'tcx> {
    #[error("non-exhaustive match expression\npattern `{0}` not covered")]
    NonexhaustiveMatch(Witness<'p, 'tcx>),
    #[error("redundant pattern")]
    RedundantPattern,
}

impl<'p, 'tcx> Deref for MatchCtxt<'p, 'tcx> {
    type Target = LoweringCtx<'tcx>;

    fn deref(&self) -> &'p Self::Target {
        self.lcx
    }
}
