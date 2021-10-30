// use serde;

use crate::types::{Changed, Delta};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub enum VecChange<Desc> {
    Added(Desc),
    Removed(Desc),
}

impl<Value: PartialEq + Delta> Delta for Vec<Value> {
    type Desc = Vec<Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter().map(|x| x.describe()).collect()
    }

    type Change = Vec<VecChange<Value::Desc>>;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        let mut changes = Vec::new();
        changes.append(
            &mut other
                .iter()
                .map(|v| {
                    if self.contains(v) {
                        Changed::Unchanged
                    } else {
                        Changed::Changed(VecChange::Added(v.describe()))
                    }
                })
                .flatten()
                .collect(),
        );
        changes.append(
            &mut self
                .iter()
                .map(|v| {
                    if !other.contains(v) {
                        Changed::Changed(VecChange::Removed(v.describe()))
                    } else {
                        Changed::Unchanged
                    }
                })
                .flatten()
                .collect(),
        );
        if changes.is_empty() {
            Changed::Unchanged
        } else {
            Changed::Changed(changes)
        }
    }
}
