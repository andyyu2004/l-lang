use lcore::queries::Queries;

mod generics;
mod inherent_impls;
mod trait_impls;
mod tys;

pub fn provide(queries: &mut Queries) {
    inherent_impls::provide(queries);
    trait_impls::provide(queries);
    generics::provide(queries);
    tys::provide(queries);
}
