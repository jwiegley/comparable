use crate::types::{Changed, Delta};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct StringChange(pub String, pub String);

impl Delta for String {
    type Desc = String;

    fn describe(&self) -> Self::Desc {
        self.to_string()
    }

    type Change = StringChange;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(StringChange(self.to_string(), other.to_string()))
        } else {
            Changed::Unchanged
        }
    }
}
