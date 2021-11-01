use crate::types::{Changed, Comparable};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct StringChange(pub String, pub String);

impl Comparable for String {
    type Desc = String;

    fn describe(&self) -> Self::Desc {
        self.to_string()
    }

    type Change = StringChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(StringChange(self.to_string(), other.to_string()))
        } else {
            Changed::Unchanged
        }
    }
}

impl Comparable for &str {
    type Desc = <String as Comparable>::Desc;

    fn describe(&self) -> Self::Desc {
        self.to_string()
    }

    type Change = <String as Comparable>::Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(StringChange(self.to_string(), other.to_string()))
        } else {
            Changed::Unchanged
        }
    }
}
