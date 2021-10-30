// use serde;

use crate::types::{Changed, Delta, EnumChange};

impl<T: Delta> Delta for Option<T> {
    type Desc = Option<T::Desc>;

    fn describe(&self) -> Self::Desc {
        match self {
            Some(x) => Some(x.describe()),
            None => None,
        }
    }

    type Change = EnumChange<Self::Desc, T::Change>;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        match (self, other) {
            (None, None) => Changed::Unchanged,
            (Some(x), Some(y)) => x.delta(y).map(EnumChange::SameVariant),
            (_, _) => Changed::Changed(EnumChange::DiffVariant(self.describe(), other.describe())),
        }
    }
}
