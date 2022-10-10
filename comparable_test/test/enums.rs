#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

use comparable::{Changed::*, *};

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

	assert_changes!(&UnitEnum::Field, &UnitEnum::Field, Unchanged);
}

#[test]
fn test_enum_1_variant_1_unnamed_field_scalar() {
	#[derive(Comparable)]
	enum ScalarEnum {
		Field(i32),
	}

	assert_changes!(&ScalarEnum::Field(100), &ScalarEnum::Field(100), Unchanged);
	assert_changes!(
		&ScalarEnum::Field(100),
		&ScalarEnum::Field(200),
		Changed(ScalarEnumChange::BothField(I32Change(100, 200))),
	);
}

#[test]
fn test_enum_2_variants_1_unnamed_field_scalar() {
	#[derive(Comparable)]
	enum ScalarEnum {
		Field1(i32),
		Field2(i32),
	}

	assert_changes!(&ScalarEnum::Field1(100), &ScalarEnum::Field1(100), Unchanged,);
	assert_changes!(
		&ScalarEnum::Field1(100),
		&ScalarEnum::Field1(200),
		Changed(ScalarEnumChange::BothField1(I32Change(100, 200))),
	);
	assert_changes!(
		&ScalarEnum::Field1(100),
		&ScalarEnum::Field2(100),
		Changed(ScalarEnumChange::Different(ScalarEnumDesc::Field1(100), ScalarEnumDesc::Field2(100),)),
	);
}

#[test]
fn test_enum_1_variant_1_named_field_scalar() {
	#[derive(Comparable)]
	enum ScalarNamedEnum {
		Field { some_int: i32 },
	}

	assert_changes!(&ScalarNamedEnum::Field { some_int: 100 }, &ScalarNamedEnum::Field { some_int: 100 }, Unchanged,);
	assert_changes!(
		&ScalarNamedEnum::Field { some_int: 100 },
		&ScalarNamedEnum::Field { some_int: 200 },
		Changed(ScalarNamedEnumChange::BothField { some_int: I32Change(100, 200) }),
	);
}

#[test]
fn test_enum_1_variant_ignored() {
	#[derive(Comparable)]
	pub enum ScalarEnumIgnore {
		#[comparable_ignore]
		Field,
	}

	assert_changes!(&ScalarEnumIgnore::Field, &ScalarEnumIgnore::Field, Unchanged,);
}

#[test]
fn test_enum_1_variant_2_named_fields_scalar() {
	#[derive(Comparable)]
	enum ScalarNamedEnum {
		Field { some_int: i32, some_ulong: u64 },
	}

	assert_changes!(
		&ScalarNamedEnum::Field { some_int: 100, some_ulong: 200 },
		&ScalarNamedEnum::Field { some_int: 100, some_ulong: 200 },
		Unchanged,
	);
	assert_changes!(
		&ScalarNamedEnum::Field { some_int: 100, some_ulong: 200 },
		&ScalarNamedEnum::Field { some_int: 200, some_ulong: 200 },
		Changed(ScalarNamedEnumChange::BothField { some_int: Changed(I32Change(100, 200)), some_ulong: Unchanged }),
	);
	assert_changes!(
		&ScalarNamedEnum::Field { some_int: 100, some_ulong: 200 },
		&ScalarNamedEnum::Field { some_int: 100, some_ulong: 300 },
		Changed(ScalarNamedEnumChange::BothField { some_int: Unchanged, some_ulong: Changed(U64Change(200, 300)) }),
	);
	assert_changes!(
		&ScalarNamedEnum::Field { some_int: 100, some_ulong: 200 },
		&ScalarNamedEnum::Field { some_int: 200, some_ulong: 300 },
		Changed(ScalarNamedEnumChange::BothField {
			some_int: Changed(I32Change(100, 200)),
			some_ulong: Changed(U64Change(200, 300)),
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

	assert_changes!(&ScalarNamedEnum::Field1 { some_int: 100 }, &ScalarNamedEnum::Field1 { some_int: 100 }, Unchanged,);
	assert_changes!(
		&ScalarNamedEnum::Field1 { some_int: 100 },
		&ScalarNamedEnum::Field1 { some_int: 200 },
		Changed(ScalarNamedEnumChange::BothField1 { some_int: I32Change(100, 200) }),
	);
	assert_changes!(
		&ScalarNamedEnum::Field1 { some_int: 100 },
		&ScalarNamedEnum::Field2 { some_int: 100 },
		Changed(ScalarNamedEnumChange::Different(
			ScalarNamedEnumDesc::Field1 { some_int: 100 },
			ScalarNamedEnumDesc::Field2 { some_int: 100 },
		)),
	);
}

#[test]
fn test_enum_5_variants() {
	#[derive(Comparable)]
	enum MyEnum {
		UnitField,
		ScalarUnnamedField(i32),
		ScalarNamedField { some_int: i32 },
		ScalarUnnamedFields(i32, u64),
		ScalarNamedFields { some_int: i32, some_ulong: u64 },
	}

	assert_changes!(&MyEnum::UnitField, &MyEnum::UnitField, Unchanged);

	assert_changes!(&MyEnum::ScalarUnnamedField(100), &MyEnum::ScalarUnnamedField(100), Unchanged,);
	assert_changes!(
		&MyEnum::ScalarUnnamedField(100),
		&MyEnum::ScalarUnnamedField(200),
		Changed(MyEnumChange::BothScalarUnnamedField(I32Change(100, 200))),
	);

	assert_changes!(
		&MyEnum::ScalarNamedField { some_int: 100 },
		&MyEnum::ScalarNamedField { some_int: 100 },
		Unchanged,
	);
	assert_changes!(
		&MyEnum::ScalarNamedField { some_int: 100 },
		&MyEnum::ScalarNamedField { some_int: 200 },
		Changed(MyEnumChange::BothScalarNamedField { some_int: I32Change(100, 200) }),
	);

	assert_changes!(&MyEnum::ScalarUnnamedFields(100, 200), &MyEnum::ScalarUnnamedFields(100, 200), Unchanged,);
	assert_changes!(
		&MyEnum::ScalarUnnamedFields(100, 200),
		&MyEnum::ScalarUnnamedFields(200, 200),
		Changed(MyEnumChange::BothScalarUnnamedFields(Changed(I32Change(100, 200)), Unchanged,)),
	);
	assert_changes!(
		&MyEnum::ScalarUnnamedFields(100, 200),
		&MyEnum::ScalarUnnamedFields(100, 300),
		Changed(MyEnumChange::BothScalarUnnamedFields(Unchanged, Changed(U64Change(200, 300)),)),
	);
	assert_changes!(
		&MyEnum::ScalarUnnamedFields(100, 200),
		&MyEnum::ScalarUnnamedFields(200, 300),
		Changed(MyEnumChange::BothScalarUnnamedFields(Changed(I32Change(100, 200)), Changed(U64Change(200, 300)),)),
	);

	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		Unchanged,
	);
	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 200, some_ulong: 200 },
		Changed(MyEnumChange::BothScalarNamedFields { some_int: Changed(I32Change(100, 200)), some_ulong: Unchanged }),
	);
	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 300 },
		Changed(MyEnumChange::BothScalarNamedFields { some_int: Unchanged, some_ulong: Changed(U64Change(200, 300)) }),
	);
	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 200, some_ulong: 300 },
		Changed(MyEnumChange::BothScalarNamedFields {
			some_int: Changed(I32Change(100, 200)),
			some_ulong: Changed(U64Change(200, 300)),
		}),
	);
}

#[test]
fn test_enum_5_variants_as_struct() {
	#[derive(Comparable)]
	#[variant_struct_fields]
	enum MyEnum {
		UnitField,
		ScalarUnnamedField(i32),
		ScalarNamedField { some_int: i32 },
		ScalarUnnamedFields(i32, u64),
		ScalarNamedFields { some_int: i32, some_ulong: u64 },
	}

	assert_changes!(&MyEnum::UnitField, &MyEnum::UnitField, Unchanged);

	assert_changes!(&MyEnum::ScalarUnnamedField(100), &MyEnum::ScalarUnnamedField(100), Unchanged,);
	assert_changes!(
		&MyEnum::ScalarUnnamedField(100),
		&MyEnum::ScalarUnnamedField(200),
		Changed(MyEnumChange::BothScalarUnnamedField(I32Change(100, 200))),
	);

	assert_changes!(
		&MyEnum::ScalarNamedField { some_int: 100 },
		&MyEnum::ScalarNamedField { some_int: 100 },
		Unchanged,
	);
	assert_changes!(
		&MyEnum::ScalarNamedField { some_int: 100 },
		&MyEnum::ScalarNamedField { some_int: 200 },
		Changed(MyEnumChange::BothScalarNamedField { some_int: I32Change(100, 200) }),
	);

	assert_changes!(&MyEnum::ScalarUnnamedFields(100, 200), &MyEnum::ScalarUnnamedFields(100, 200), Unchanged,);
	assert_changes!(
		&MyEnum::ScalarUnnamedFields(100, 200),
		&MyEnum::ScalarUnnamedFields(200, 200),
		Changed(MyEnumChange::BothScalarUnnamedFields(vec![MyEnumScalarUnnamedFieldsChange::Field0(I32Change(
			100, 200
		)),])),
	);
	assert_changes!(
		&MyEnum::ScalarUnnamedFields(100, 200),
		&MyEnum::ScalarUnnamedFields(100, 300),
		Changed(MyEnumChange::BothScalarUnnamedFields(vec![MyEnumScalarUnnamedFieldsChange::Field1(U64Change(
			200, 300
		)),])),
	);
	assert_changes!(
		&MyEnum::ScalarUnnamedFields(100, 200),
		&MyEnum::ScalarUnnamedFields(200, 300),
		Changed(MyEnumChange::BothScalarUnnamedFields(vec![
			MyEnumScalarUnnamedFieldsChange::Field0(I32Change(100, 200)),
			MyEnumScalarUnnamedFieldsChange::Field1(U64Change(200, 300)),
		])),
	);

	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		Unchanged,
	);
	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 200, some_ulong: 200 },
		Changed(MyEnumChange::BothScalarNamedFields(
			vec![MyEnumScalarNamedFieldsChange::SomeInt(I32Change(100, 200)),]
		)),
	);
	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 300 },
		Changed(MyEnumChange::BothScalarNamedFields(vec![MyEnumScalarNamedFieldsChange::SomeUlong(U64Change(
			200, 300
		)),])),
	);
	assert_changes!(
		&MyEnum::ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&MyEnum::ScalarNamedFields { some_int: 200, some_ulong: 300 },
		Changed(MyEnumChange::BothScalarNamedFields(vec![
			MyEnumScalarNamedFieldsChange::SomeInt(I32Change(100, 200)),
			MyEnumScalarNamedFieldsChange::SomeUlong(U64Change(200, 300)),
		])),
	);
}

#[test]
fn test_enum_fields_varying_visibility() {
	#[derive(Comparable)]
	pub enum VisiblePub {
		Field { int: i32 },
	}

	#[derive(Comparable)]
	pub(crate) enum VisiblePubCrate {
		Field { int: i32 },
	}

	#[derive(Comparable)]
	enum VisiblePrivate {
		Field { int: i32 },
	}

	assert_changes!(
		&VisiblePub::Field { int: 1 },
		&VisiblePub::Field { int: 4 },
		Changed(VisiblePubChange::BothField { int: I32Change(1, 4) }),
	);

	assert_changes!(
		&VisiblePubCrate::Field { int: 1 },
		&VisiblePubCrate::Field { int: 4 },
		Changed(VisiblePubCrateChange::BothField { int: I32Change(1, 4) }),
	);

	assert_changes!(
		&VisiblePrivate::Field { int: 1 },
		&VisiblePrivate::Field { int: 4 },
		Changed(VisiblePrivateChange::BothField { int: I32Change(1, 4) }),
	);
}
