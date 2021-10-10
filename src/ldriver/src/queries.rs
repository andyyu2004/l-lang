use crate::passes;
use lcore::queries::{Queries, QueryCtx};

pub(crate) fn query_ctx<'tcx>() -> QueryCtx<'tcx> {
    let mut queries = Queries::default();

    passes::provide(&mut queries);
    typeck::provide(&mut queries);
    mirgen::provide(&mut queries);
    mir::provide(&mut queries);
    lcore::provide(&mut queries);
    codegen::provide(&mut queries);

    QueryCtx::new(queries)
}
