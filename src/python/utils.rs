/// Implement `From` conversions between the core and python types.
#[macro_export]
macro_rules! impl_py_conversions {
    ( $core:ty, $py:ty) => {
        impl From<$py> for $core {
            fn from(value: $py) -> Self {
                value.0
            }
        }
        impl From<$core> for $py {
            fn from(value: $core) -> Self {
                Self(value)
            }
        }
    };
}
