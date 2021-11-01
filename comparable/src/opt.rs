// use serde;

use crate::types::{Changed, Comparable, EnumChange};

impl<T: Comparable> Comparable for Option<T> {
    type Desc = Option<T::Desc>;

    fn describe(&self) -> Self::Desc {
        self.as_ref().map(|x| x.describe())
    }

    type Change = EnumChange<Self::Desc, T::Change>;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        match (self, other) {
            (None, None) => Changed::Unchanged,
            (Some(x), Some(y)) => x.comparison(y).map(EnumChange::SameVariant),
            (_, _) => Changed::Changed(EnumChange::DiffVariant(self.describe(), other.describe())),
        }
    }
}
