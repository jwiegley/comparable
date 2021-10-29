use serde;

use crate::types::Delta;

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum OptionChange<Desc, Change> {
    NoneToSome(Desc),
    SomeToNone(Desc),
    Some(Change),
}

impl<T: Delta> Delta for Option<T> {
    type Desc = Option<T::Desc>;

    fn describe(&self) -> Self::Desc {
        self.as_ref().map(|x| x.describe())
    }

    type Change = OptionChange<T::Desc, T::Change>;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        match (self, other) {
            (None, None) => None,
            (Some(x), None) => Some(OptionChange::SomeToNone(x.describe())),
            (None, Some(y)) => Some(OptionChange::SomeToNone(y.describe())),
            (Some(x), Some(y)) => x.delta(y).map(OptionChange::Some),
        }
    }
}
