// use serde;

use crate::types::{Changed, Comparable};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub enum OptionChange<Desc, Change> {
    BothSome(Change),
    Different(Desc, Desc),
}

impl<T: Comparable> Comparable for Option<T> {
    type Desc = Option<T::Desc>;

    fn describe(&self) -> Self::Desc {
        self.as_ref().map(|x| x.describe())
    }

    type Change = OptionChange<Self::Desc, T::Change>;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        match (self, other) {
            (None, None) => Changed::Unchanged,
            (Some(x), Some(y)) => x.comparison(y).map(OptionChange::BothSome),
            (_, _) => Changed::Changed(OptionChange::Different(self.describe(), other.describe())),
        }
    }
}
