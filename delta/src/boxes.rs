use crate::types::{Changed, Delta};

impl<T: Delta> Delta for Box<T> {
    type Desc = T::Desc;

    fn describe(&self) -> Self::Desc {
        self.as_ref().describe()
    }

    type Change = T::Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        self.as_ref().delta(&*other)
    }
}
