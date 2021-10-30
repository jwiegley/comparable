use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;
// use serde;

use crate::types::{Changed, Delta};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub enum VecChange<Desc, Change> {
    Added(Desc),
    Change(usize, Change),
    Removed(Desc),
}

impl<Value: PartialEq + Delta> Delta for Vec<Value> {
    type Desc = Vec<Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter().map(|x| x.describe()).collect()
    }

    type Change = Vec<VecChange<Value::Desc, Value::Change>>;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        let mut changes = Vec::new();
        let other_len = other.len();
        for i in 0..self.len() {
            if i >= other_len {
                changes.push(VecChange::Removed(self[i].describe()));
            } else if let Changed::Changed(change) = self[i].delta(&other[i]) {
                changes.push(VecChange::Change(i, change));
            }
        }
        if other.len() > self.len() {
            for i in self.len()..other.len() {
                changes.push(VecChange::Added(other[i].describe()));
            }
        }
        if changes.is_empty() {
            Changed::Unchanged
        } else {
            Changed::Changed(changes)
        }
    }
}

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

impl<Value: std::hash::Hash + Ord + Delta> Delta for HashSet<Value> {
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
                .collect::<BTreeSet<&Value>>()
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
