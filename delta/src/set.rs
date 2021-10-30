// use serde;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;

use crate::types::{Changed, Delta};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub enum SetChange<Desc> {
    Added(Desc),
    Removed(Desc),
}

impl<Value: Ord + Delta> Delta for BTreeSet<Value> {
    type Desc = Vec<Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter().map(|v| v.describe()).collect()
    }

    type Change = Vec<SetChange<Value::Desc>>;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        let mut changes = Vec::new();
        changes.append(
            &mut other
                .iter()
                .map(|v| {
                    if self.contains(v) {
                        Changed::Unchanged
                    } else {
                        Changed::Changed(SetChange::Added(v.describe()))
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
                        Changed::Changed(SetChange::Removed(v.describe()))
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

impl<Value: std::hash::Hash + Eq + Delta> Delta for HashSet<Value> {
    type Desc = Vec<Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter().map(|v| v.describe()).collect()
    }

    type Change = Vec<SetChange<Value::Desc>>;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        let mut changes = Vec::new();
        changes.append(
            &mut other
                .iter()
                .map(|v| {
                    if self.contains(v) {
                        Changed::Unchanged
                    } else {
                        Changed::Changed(SetChange::Added(v.describe()))
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
                        Changed::Changed(SetChange::Removed(v.describe()))
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
