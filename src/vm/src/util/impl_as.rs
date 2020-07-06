#[macro_export]
macro_rules! impl_into {
    ($item:path, $variant:ident, $ret_ty:ty) => {
        impl Into<$ret_ty> for $item {
            fn into(self) -> $ret_ty {
                match self {
                    Self::$variant(x) => x,
                    _ => panic!(
                        "expected `{}`, found `{:?}`",
                        std::any::type_name::<$ret_ty>(),
                        self
                    ),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_as_mut {
    ($item:path, $fnname:ident, $variant:ident, $ret_ty:ty) => {
        impl $item {
            pub fn $fnname(&mut self) -> &mut $ret_ty {
                match self {
                    Self::$variant(x) => x,
                    _ => panic!(
                        "expected `{}`, found `{:?}`",
                        std::any::type_name::<$ret_ty>(),
                        self
                    ),
                }
            }
        }
    };
}
