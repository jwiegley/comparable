use std::collections::{BTreeMap, HashMap};

use delta::*;
use delta_derive::Delta;

// This 'Foo' is provided to show what #[derive(Delta)] will expand into when
// applied to the 'Bar' type below.

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

#[derive(Delta)]
struct Bar {
    alpha: bool,
    beta: Vec<bool>,
    gamma: HashMap<u64, bool>,
    delta: BTreeMap<u64, Bar>,
}

impl Default for Bar {
    fn default() -> Self {
        Bar {
            alpha: false,
            beta: Vec::new(),
            gamma: HashMap::new(),
            delta: BTreeMap::new(),
        }
    }
}

#[test]
fn test_delta_bar() {
    let mut x = Bar::default();
    x.alpha = true;
    x.beta.push(true);
    x.gamma.insert(10, true);
    let mut y = Bar::default();
    y.alpha = false;
    x.gamma.insert(10, true);
    x.gamma.insert(20, false);
    assert_changes(
        &x,
        &y,
        Some(vec![
            BarChange::Alpha(BoolChange(true, false)),
            BarChange::Beta(vec![VecChange::Removed(true)]),
            BarChange::Gamma(vec![MapChange::Removed(10), MapChange::Removed(20)]),
        ]),
    );
}
