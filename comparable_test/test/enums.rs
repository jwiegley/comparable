#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

// use std::collections::{BTreeMap, HashMap, HashSet};

use comparable::*;

#[test]
fn test_enum_0_variants() {
    #[derive(Comparable)]
    enum Unit {}

    // These can never be instantianted.
}

#[test]
fn test_enum_1_variant_0_fields() {
    #[derive(Comparable)]
    enum UnitEnum {
        Field,
    }

    assert_changes(&UnitEnum::Field, &UnitEnum::Field, Changed::Unchanged);
}

#[test]
fn test_enum_1_variant_1_unnamed_field_scalar() {
    #[derive(Comparable)]
    enum ScalarEnum {
        Field(i32),
    }

    assert_changes(
        &ScalarEnum::Field(100),
        &ScalarEnum::Field(100),
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarEnum::Field(100),
        &ScalarEnum::Field(200),
        Changed::Changed(ScalarEnumChange::BothField(I32Change(100, 200))),
    );
}

#[test]
fn test_enum_2_variants_1_unnamed_field_scalar() {
    #[derive(Comparable)]
    enum ScalarEnum {
        Field1(i32),
        Field2(i32),
    }

    assert_changes(
        &ScalarEnum::Field1(100),
        &ScalarEnum::Field1(100),
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarEnum::Field1(100),
        &ScalarEnum::Field1(200),
        Changed::Changed(ScalarEnumChange::BothField1(I32Change(100, 200))),
    );
    assert_changes(
        &ScalarEnum::Field1(100),
        &ScalarEnum::Field2(100),
        Changed::Changed(ScalarEnumChange::Different(
            ScalarEnumDesc::Field1(100),
            ScalarEnumDesc::Field2(100),
        )),
    );
}

#[test]
fn test_enum_1_variant_1_named_field_scalar() {
    #[derive(Comparable)]
    enum ScalarNamedEnum {
        Field { some_int: i32 },
    }

    assert_changes(
        &ScalarNamedEnum::Field { some_int: 100 },
        &ScalarNamedEnum::Field { some_int: 100 },
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarNamedEnum::Field { some_int: 100 },
        &ScalarNamedEnum::Field { some_int: 200 },
        Changed::Changed(ScalarNamedEnumChange::BothField {
            some_int: I32Change(100, 200),
        }),
    );
}

#[test]
fn test_enum_1_variant_2_named_fields_scalar() {
    #[derive(Comparable)]
    enum ScalarNamedEnum {
        Field { some_int: i32, some_ulong: u64 },
    }

    assert_changes(
        &ScalarNamedEnum::Field {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedEnum::Field {
            some_int: 100,
            some_ulong: 200,
        },
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarNamedEnum::Field {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedEnum::Field {
            some_int: 200,
            some_ulong: 200,
        },
        Changed::Changed(ScalarNamedEnumChange::BothField {
            some_int: Changed::Changed(I32Change(100, 200)),
            some_ulong: Changed::Unchanged,
        }),
    );
    assert_changes(
        &ScalarNamedEnum::Field {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedEnum::Field {
            some_int: 100,
            some_ulong: 300,
        },
        Changed::Changed(ScalarNamedEnumChange::BothField {
            some_int: Changed::Unchanged,
            some_ulong: Changed::Changed(U64Change(200, 300)),
        }),
    );
    assert_changes(
        &ScalarNamedEnum::Field {
            some_int: 100,
            some_ulong: 200,
        },
        &ScalarNamedEnum::Field {
            some_int: 200,
            some_ulong: 300,
        },
        Changed::Changed(ScalarNamedEnumChange::BothField {
            some_int: Changed::Changed(I32Change(100, 200)),
            some_ulong: Changed::Changed(U64Change(200, 300)),
        }),
    );
}

#[test]
fn test_enum_2_variants_1_named_fields_scalar() {
    #[derive(Comparable)]
    enum ScalarNamedEnum {
        Field1 { some_int: i32 },
        Field2 { some_int: i32 },
    }

    assert_changes(
        &ScalarNamedEnum::Field1 { some_int: 100 },
        &ScalarNamedEnum::Field1 { some_int: 100 },
        Changed::Unchanged,
    );
    assert_changes(
        &ScalarNamedEnum::Field1 { some_int: 100 },
        &ScalarNamedEnum::Field1 { some_int: 200 },
        Changed::Changed(ScalarNamedEnumChange::BothField1 {
            some_int: I32Change(100, 200),
        }),
    );
    assert_changes(
        &ScalarNamedEnum::Field1 { some_int: 100 },
        &ScalarNamedEnum::Field2 { some_int: 100 },
        Changed::Changed(ScalarNamedEnumChange::Different(
            ScalarNamedEnumDesc::Field1 { some_int: 100 },
            ScalarNamedEnumDesc::Field2 { some_int: 100 },
        )),
    );
}
