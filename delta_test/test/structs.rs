#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

// use std::collections::{BTreeMap, HashMap, HashSet};

use delta::*;

#[test]
fn test_unit_struct() {
    #[derive(Delta)]
    struct Unit;

    assert_changes(&Unit, &Unit, Changed::Unchanged);
}

#[test]
fn test_unnamed_singleton_struct_unit_field() {
    #[derive(Delta)]
    struct UnitField(());

    assert_changes(&UnitField(()), &UnitField(()), Changed::Unchanged);
}

#[test]
fn test_unnamed_singleton_struct_scalar_field() {
    #[derive(Delta)]
    struct ScalarField(i32);

    assert_changes(&ScalarField(100), &ScalarField(100), Changed::Unchanged);
    assert_changes(
        &ScalarField(100),
        &ScalarField(200),
        Changed::Changed(ScalarFieldChange(I32Change(100, 200))),
    );
}

/*
#[test]
fn test_unnamed_singleton_struct_vec_field() {}

#[test]
fn test_unnamed_singleton_struct_struct_field() {}

#[test]
fn test_named_singleton_struct_unit_field() {}

#[test]
fn test_named_singleton_struct_scalar_field() {}

#[test]
fn test_named_singleton_struct_vec_field() {}

#[test]
fn test_named_singleton_struct_struct_field() {}

#[test]
fn test_unnamed_struct_unit_fields() {}
*/

#[test]
fn test_unnamed_struct_scalar_fields() {
    #[derive(Delta)]
    struct ScalarFields(i32, u64);

    assert_changes(
        &ScalarFields(100, 200),
        &ScalarFields(100, 200),
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarFields(100, 200),
        &ScalarFields(200, 200),
        Changed::Changed(vec![ScalarFieldsChange::Field0(I32Change(100, 200))]),
    );
    assert_changes(
        &ScalarFields(100, 200),
        &ScalarFields(100, 300),
        Changed::Changed(vec![ScalarFieldsChange::Field1(U64Change(200, 300))]),
    );
    assert_changes(
        &ScalarFields(100, 200),
        &ScalarFields(200, 300),
        Changed::Changed(vec![
            ScalarFieldsChange::Field0(I32Change(100, 200)),
            ScalarFieldsChange::Field1(U64Change(200, 300)),
        ]),
    );
}

/*
#[test]
fn test_unnamed_struct_vec_fields() {}

#[test]
fn test_unnamed_struct_struct_fields() {}

#[test]
fn test_unnamed_struct_mixed_fields() {}

#[test]
fn test_named_struct_unit_fields() {}
*/

#[test]
fn test_named_struct_scalar_fields() {
    #[derive(Delta)]
    struct ScalarNamedFields {
        some_int: i32,
        some_ulong: u64,
    }

    assert_changes(
        &ScalarNamedFields {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedFields {
            some_int: 100,
            some_ulong: 200,
        },
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarNamedFields {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedFields {
            some_int: 200,
            some_ulong: 200,
        },
        Changed::Changed(vec![ScalarNamedFieldsChange::SomeInt(I32Change(100, 200))]),
    );
    assert_changes(
        &ScalarNamedFields {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedFields {
            some_int: 100,
            some_ulong: 300,
        },
        Changed::Changed(vec![ScalarNamedFieldsChange::SomeUlong(U64Change(
            200, 300,
        ))]),
    );
    assert_changes(
        &ScalarNamedFields {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedFields {
            some_int: 200,
            some_ulong: 300,
        },
        Changed::Changed(vec![
            ScalarNamedFieldsChange::SomeInt(I32Change(100, 200)),
            ScalarNamedFieldsChange::SomeUlong(U64Change(200, 300)),
        ]),
    );
}

/*
#[test]
fn test_named_struct_vec_fields() {}

#[test]
fn test_named_struct_struct_fields() {}

#[test]
fn test_named_struct_mixed_fields() {}

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

struct MyEnumOneChange(<bool as Delta>::Change);

struct MyEnumTwoChange {
    two: <Vec<bool> as Delta>::Change,
}

struct MyEnumThreeChange(<Bar as Delta>::Change);

struct MyEnumFourChange;

enum MyEnumChange_ {
    One(MyEnumOneChange),
    Two(MyEnumTwoChange),
    Three(MyEnumThreeChange),
    Four(MyEnumFourChange),
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

#[derive(Delta)]
#[describe_type(())]
#[describe_body(())]
struct Qalux(bool);

impl Default for Qalux {
    fn default() -> Self {
        Qalux(true)
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
        Changed::Changed(vec![VecChange::Changed(0, I32Change(100, 200))]),
    );
    assert_changes(
        &vec![],
        &vec![100],
        Changed::Changed(vec![VecChange::Added(0, 100)]),
    );
    assert_changes(
        &vec![100],
        &vec![],
        Changed::Changed(vec![VecChange::Removed(0, 100)]),
    );
    assert_changes(
        &vec![100, 200, 300],
        &vec![100, 400, 300],
        Changed::Changed(vec![VecChange::Changed(1, I32Change(200, 400))]),
    );

    assert_changes(
        &HashSet::from(vec![100, 200, 300].into_iter().collect()),
        &HashSet::from(vec![100, 400, 300].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(400), SetChange::Removed(200)]),
    );

    let desc = Bar::default().describe();
    println!("Bar::default().describe() = {:?}", desc);

    assert_changes(
        &Qalux(true),
        &Qalux(false),
        Changed::Changed(QaluxChange(BoolChange(true, false))),
    );
}
*/
