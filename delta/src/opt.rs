// use serde;

use crate::types::{Changed, Delta, EnumChange};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub enum OptionDesc<Desc> {
    Some(Desc),
    None,
}

impl<T: Delta> Delta for Option<T> {
    type Desc = OptionDesc<T::Desc>;

    fn describe(&self) -> Self::Desc {
        match self {
            Some(x) => OptionDesc::Some(x.describe()),
            None => OptionDesc::None,
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
