use lcore::queries::Queries;

mod generics;
mod inherent;
mod tys;

pub fn provide(queries: &mut Queries) {
    inherent::provide(queries);
    generics::provide(queries);
    tys::provide(queries);
}
