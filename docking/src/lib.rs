#![no_std]
pub mod reports;


macro_rules! enum_from_variants {
    ($($s:ident)*) => {
        $(
            impl From<$s> for Report {
                fn from(value: $s) -> Report {
                    Report::$s(value)
                }
            }
        )*
    };
}