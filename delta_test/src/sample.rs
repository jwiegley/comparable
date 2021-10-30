use std::collections::{BTreeMap, HashMap, HashSet};

use delta::*;

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

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
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

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
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
            Changed::Unchanged
        } else {
            Changed::Changed(changes)
        }
    }
}

#[derive(Delta)]
enum MyEnum {
    One(bool),
    Two { two: Vec<bool> },
    Three(Bar),
    Four,
}

impl Default for MyEnum {
    fn default() -> Self {
        MyEnum::Four
    }
}

#[derive(Delta)]
#[compare_default = true]
struct Bar {
    alpha: bool,
    #[delta_ignore]
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

#[derive(Delta)]
#[compare_default = true]
struct Baz(bool, bool);

impl Default for Baz {
    fn default() -> Self {
        Baz(false, false)
    }
}

#[derive(Delta)]
#[describe_type(())]
#[describe_body(())]
struct Quux();

impl Default for Quux {
    fn default() -> Self {
        Quux()
    }
}

#[derive(Delta, PartialEq, Debug, Clone)]
struct Empty;

impl Default for Empty {
    fn default() -> Self {
        Empty
    }
}

#[test]
fn test_delta_bar() {
    let mut x1 = Bar {
        alpha: true,
        ..Bar::default()
    };
    x1.beta.push(true);
    x1.gamma.insert(10, true);
    let y1 = Bar::default();
    x1.gamma.insert(10, true);
    x1.gamma.insert(20, false);

    assert_changes(
        &MyEnum::Three(x1),
        &MyEnum::Three(y1),
        Changed::Changed(EnumChange::SameVariant(MyEnumChange::Three(
            Changed::Changed(vec![
                BarChange::Alpha(BoolChange(true, false)),
                // Change doesn't appear because we use #[delta_ignore] above
                // BarChange::Beta(vec![VecChange::Removed(true)]),
                BarChange::Gamma(vec![MapChange::Removed(10), MapChange::Removed(20)]),
            ]),
        ))),
    );

    assert_changes(
        &MyEnum::One(true),
        &MyEnum::Two { two: vec![false] },
        Changed::Changed(EnumChange::DiffVariant(
            MyEnumDesc::One(true),
            MyEnumDesc::Two { two: vec![false] },
        )),
    );

    assert_changes(
        &MyEnum::One(true),
        &MyEnum::One(false),
        Changed::Changed(EnumChange::SameVariant(MyEnumChange::One(
            Changed::Changed(BoolChange(true, false)),
        ))),
    );

    assert_changes(
        &MyEnum::One(true),
        &MyEnum::Four,
        Changed::Changed(EnumChange::DiffVariant(
            MyEnumDesc::One(true),
            MyEnumDesc::Four,
        )),
    );

    let x2 = Baz::default();
    let y2 = Baz::default();
    assert_changes(&x2, &y2, Changed::Unchanged);

    let x3 = Quux::default();
    let y3 = Quux::default();
    assert_changes(&x3, &y3, Changed::Unchanged);

    assert_changes(&100, &100, Changed::Unchanged);
    assert_changes(&100, &200, Changed::Changed(I32Change(100, 200)));
    assert_changes(&true, &false, Changed::Changed(BoolChange(true, false)));
    assert_changes(
        &"foo",
        &"bar",
        Changed::Changed(StringChange("foo".to_string(), "bar".to_string())),
    );

    assert_changes(&vec![100], &vec![100], Changed::Unchanged);
    assert_changes(
        &vec![100],
        &vec![200],
        Changed::Changed(vec![VecChange::Change(0, I32Change(100, 200))]),
    );
    assert_changes(
        &vec![],
        &vec![100],
        Changed::Changed(vec![VecChange::Added(100)]),
    );
    assert_changes(
        &vec![100],
        &vec![],
        Changed::Changed(vec![VecChange::Removed(100)]),
    );
    assert_changes(
        &vec![100, 200, 300],
        &vec![100, 400, 300],
        Changed::Changed(vec![VecChange::Change(1, I32Change(200, 400))]),
    );

    assert_changes(
        &HashSet::from(vec![100, 200, 300].into_iter().collect()),
        &HashSet::from(vec![100, 400, 300].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(400), SetChange::Removed(200)]),
    );

    let desc = Bar::default().describe();
    println!("Bar::default().describe() = {:?}", desc);
}
