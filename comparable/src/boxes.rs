use crate::types::{Changed, Comparable};

impl<T: Comparable> Comparable for Box<T> {
    type Desc = T::Desc;

    fn describe(&self) -> Self::Desc {
        self.as_ref().describe()
    }

    type Change = T::Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        self.as_ref().comparison(&*other)
    }
}
