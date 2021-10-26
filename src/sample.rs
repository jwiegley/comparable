use std::collections::{BTreeMap, HashMap};

use crate::types::Delta;

struct Foo {
    alpha: bool,
    beta: Vec<bool>,
    gamma: HashMap<u64, bool>,
    delta: BTreeMap<u64, Foo>,
}

impl Default for Foo {
    fn default() -> Self {
        Foo {
            alpha: false,
            beta: Vec::new(),
            gamma: HashMap::new(),
            delta: BTreeMap::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
enum FooChange {
    Alpha(<bool as Delta>::Change),
    Beta(<Vec<bool> as Delta>::Change),
    Gamma(<HashMap<u64, bool> as Delta>::Change),
    Delta(<BTreeMap<u64, Foo> as Delta>::Change),
}

impl Delta for Foo {
    type Desc = Vec<FooChange>;

    fn describe(&self) -> Self::Desc {
        Foo::default().delta(self).unwrap_or_default()
    }

    type Change = Vec<FooChange>;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        let changes: Vec<FooChange> = vec![
            self.alpha.delta(&other.alpha).map(FooChange::Alpha),
            self.beta.delta(&other.beta).map(FooChange::Beta),
            self.gamma.delta(&other.gamma).map(FooChange::Gamma),
            self.delta.delta(&other.delta).map(FooChange::Delta),
        ]
        .into_iter()
        .flatten()
        .collect();
        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }
}
