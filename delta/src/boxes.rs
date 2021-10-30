use crate::types::{Changed, Delta};

impl<T: Delta> Delta for Box<T> {
    type Desc = Box<T::Desc>;

    fn describe(&self) -> Self::Desc {
        Box::new(self.as_ref().describe())
    }

    type Change = Box<T::Change>;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        self.as_ref().delta(&*other).map(Box::new)
    }
}
