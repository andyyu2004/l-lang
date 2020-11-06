use lcore::queries::{Queries, QueryCtx};

fn construct_queries() -> Queries {
    let mut queries = Queries::default();
    typeck::provide(&mut queries);

    queries.assert_is_fully_populated();
    queries
}

crate fn queries<'tcx>() -> QueryCtx<'tcx> {
    QueryCtx::new(construct_queries())
}
