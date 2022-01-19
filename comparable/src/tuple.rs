use crate::types::{Changed, Comparable};

macro_rules! impl_all {
    ( $a: ty) => {
        comparable_helper::impl_comparable_for_tuple!(($a,));
    };
    ( $b: ty, $($a: ty),+ ) => {
        comparable_helper::impl_comparable_for_tuple!(($b,$($a),+));
        impl_all!($($a),*);
    };
}

impl_all!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
