#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

// use std::collections::{BTreeMap, HashMap, HashSet};

use comparable::*;

#[test]
fn test_struct_0_fields() {
    #[derive(Comparable)]
    struct Unit;

    assert_changes(&Unit, &Unit, Changed::Unchanged);
}

#[test]
fn test_struct_1_unnamed_field_unit() {
    #[derive(Comparable)]
    struct UnitField(());

    assert_changes(&UnitField(()), &UnitField(()), Changed::Unchanged);
}

#[test]
fn test_struct_1_unnamed_field_scalar() {
    #[derive(Comparable)]
    struct ScalarField(i32);

    assert_changes(&ScalarField(100), &ScalarField(100), Changed::Unchanged);
    assert_changes(
        &ScalarField(100),
        &ScalarField(200),
        Changed::Changed(ScalarFieldChange(I32Change(100, 200))),
    );
}

#[test]
fn test_struct_2_unnamed_fields_scalar() {
    #[derive(Comparable)]
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

#[test]
fn test_struct_1_unnamed_field_ignored() {
    #[derive(Comparable)]
    pub struct ScalarUnnamedVecIgnored(#[comparable_ignore] pub Vec<u8>);

    assert_changes(
        &ScalarUnnamedVecIgnored(Vec::new()),
        &ScalarUnnamedVecIgnored(Vec::new()),
        Changed::Unchanged,
    );
}

#[test]
fn test_struct_1_unnamed_field_ignored_with_attrs() {
    #[derive(Comparable)]
    #[describe_type(String)]
    #[describe_body(self.to_string())]
    pub struct ScalarUnnamedVecIgnored(#[comparable_ignore] pub Vec<u8>);

    impl ToString for ScalarUnnamedVecIgnored {
        fn to_string(&self) -> String {
            "it's a vector".to_string()
        }
    }

    assert_changes(
        &ScalarUnnamedVecIgnored(Vec::new()),
        &ScalarUnnamedVecIgnored(Vec::new()),
        Changed::Unchanged,
    );
}

#[test]
fn test_struct_2_named_fields_scalar() {
    #[derive(Comparable)]
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

#[test]
fn test_struct_1_named_field_ignored() {
    #[derive(Comparable)]
    pub struct ScalarNamedVecIgnored {
        #[comparable_ignore]
        pub some_ints: Vec<u8>,
    }

    assert_changes(
        &ScalarNamedVecIgnored {
            some_ints: Vec::new(),
        },
        &ScalarNamedVecIgnored {
            some_ints: Vec::new(),
        },
        Changed::Unchanged,
    );
}

#[test]
fn test_struct_1_named_field_not_ignored() {
    #[derive(Comparable)]
    pub struct ScalarNamedVecNotIgnored {
        pub some_ints: Vec<u8>,
    }

    assert_changes(
        &ScalarNamedVecNotIgnored {
            some_ints: Vec::new(),
        },
        &ScalarNamedVecNotIgnored {
            some_ints: Vec::new(),
        },
        Changed::Unchanged,
    );
}
