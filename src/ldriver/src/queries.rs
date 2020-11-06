use lcore::queries::{Queries, QueryCtx};

crate fn query_ctx<'tcx>() -> QueryCtx<'tcx> {
    let mut queries = Queries::default();

    typeck::provide(&mut queries);

    QueryCtx::new(queries)
}
