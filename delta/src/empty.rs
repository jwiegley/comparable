use crate::types::{Changed, Delta};

impl<T> Delta for std::iter::Empty<T> {
    type Desc = ();

    fn describe(&self) -> Self::Desc {
        ()
    }

    type Change = ();

    fn delta(&self, _other: &Self) -> Changed<Self::Change> {
        Changed::Unchanged
    }
}
