use synstructure::decl_derive;

mod type_foldable;

decl_derive!([TypeFoldable, attributes(type_foldable)] => type_foldable::type_foldable_derive);
