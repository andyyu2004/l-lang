use crate::passes;
use lc_core::queries::{Queries, QueryCtx};

pub(crate) fn query_ctx<'tcx>() -> QueryCtx<'tcx> {
    let mut queries = Queries::default();

    passes::provide(&mut queries);
    lc_typeck::provide(&mut queries);
    lc_mirgen::provide(&mut queries);
    lc_mir::provide(&mut queries);
    lc_core::provide(&mut queries);
    lc_codegen::provide(&mut queries);

    QueryCtx::new(queries)
}
