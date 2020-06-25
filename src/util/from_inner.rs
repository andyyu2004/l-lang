#[macro_export]
macro_rules! impl_from_inner {
    ($inner:ty, $item:path, $variant:ident) => {
        impl From<$inner> for $item {
            fn from(x: $inner) -> Self {
                Self::$variant(x)
            }
        }
    };
}
