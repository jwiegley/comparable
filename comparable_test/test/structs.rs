#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

// use std::collections::{BTreeMap, HashMap, HashSet};

use comparable::{Changed::*, *};

#[test]
fn test_struct_0_fields() {
	#[derive(Comparable)]
	struct Unit;

	assert_changes!(&Unit, &Unit, Unchanged);
}

#[test]
fn test_struct_1_unnamed_field_unit() {
	#[derive(Comparable)]
	struct UnitField(());

	assert_changes!(&UnitField(()), &UnitField(()), Unchanged);
}

#[test]
fn test_struct_1_unnamed_field_scalar() {
	#[derive(Comparable)]
	struct ScalarField(i32);

	assert_changes!(&ScalarField(100), &ScalarField(100), Unchanged);
	assert_changes!(&ScalarField(100), &ScalarField(200), Changed(ScalarFieldChange(I32Change(100, 200))),);
}

#[test]
fn test_struct_2_unnamed_fields_scalar() {
	#[derive(Comparable)]
	struct ScalarFields(i32, u64);

	assert_changes!(&ScalarFields(100, 200), &ScalarFields(100, 200), Unchanged);
	assert_changes!(
		&ScalarFields(100, 200),
		&ScalarFields(200, 200),
		Changed(vec![ScalarFieldsChange::Field0(I32Change(100, 200))]),
	);
	assert_changes!(
		&ScalarFields(100, 200),
		&ScalarFields(100, 300),
		Changed(vec![ScalarFieldsChange::Field1(U64Change(200, 300))]),
	);
	assert_changes!(
		&ScalarFields(100, 200),
		&ScalarFields(200, 300),
		Changed(
			vec![ScalarFieldsChange::Field0(I32Change(100, 200)), ScalarFieldsChange::Field1(U64Change(200, 300)),]
		),
	);
}

#[test]
fn test_struct_1_unnamed_field_ignored() {
	#[derive(Comparable)]
	pub struct ScalarUnnamedVecIgnored(#[comparable_ignore] pub Vec<u8>);

	assert_changes!(&ScalarUnnamedVecIgnored(Vec::new()), &ScalarUnnamedVecIgnored(Vec::new()), Unchanged,);
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

	assert_changes!(&ScalarUnnamedVecIgnored(Vec::new()), &ScalarUnnamedVecIgnored(Vec::new()), Unchanged,);
}

#[test]
fn test_struct_2_named_fields_scalar() {
	#[derive(Comparable)]
	struct ScalarNamedFields {
		some_int: i32,
		some_ulong: u64,
	}

	assert_changes!(
		&ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&ScalarNamedFields { some_int: 100, some_ulong: 200 },
		Unchanged,
	);
	assert_changes!(
		&ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&ScalarNamedFields { some_int: 200, some_ulong: 200 },
		Changed(vec![ScalarNamedFieldsChange::SomeInt(I32Change(100, 200))]),
	);
	assert_changes!(
		&ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&ScalarNamedFields { some_int: 100, some_ulong: 300 },
		Changed(vec![ScalarNamedFieldsChange::SomeUlong(U64Change(200, 300,))]),
	);
	assert_changes!(
		&ScalarNamedFields { some_int: 100, some_ulong: 200 },
		&ScalarNamedFields { some_int: 200, some_ulong: 300 },
		Changed(vec![
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

	assert_changes!(
		&ScalarNamedVecIgnored { some_ints: Vec::new() },
		&ScalarNamedVecIgnored { some_ints: Vec::new() },
		Unchanged,
	);
}

#[test]
fn test_struct_1_named_field_not_ignored() {
	#[derive(Comparable)]
	pub struct ScalarNamedVecNotIgnored {
		pub some_ints: Vec<u8>,
	}

	assert_changes!(
		&ScalarNamedVecNotIgnored { some_ints: Vec::new() },
		&ScalarNamedVecNotIgnored { some_ints: Vec::new() },
		Unchanged,
	);
}

#[test]
fn test_struct_1_named_field_self_describing() {
	#[derive(Comparable, Clone, PartialEq, Debug)]
	#[self_describing]
	pub struct ScalarNamedVecNotIgnored {
		pub some_ints: Vec<u8>,
	}

	assert_changes!(
		&ScalarNamedVecNotIgnored { some_ints: vec![100] },
		&ScalarNamedVecNotIgnored { some_ints: vec![200] },
		Changed(ScalarNamedVecNotIgnoredChange { some_ints: vec![VecChange::Changed(0, U8Change(100, 200))] }),
	);
}

#[test]
fn test_struct_1_named_field_comparable_change_suffix() {
	#[derive(Comparable, Clone, PartialEq, Debug)]
	#[comparable_change_suffix(Mutated)]
	pub struct ScalarNamedVecNotIgnored {
		pub some_ints: Vec<u8>,
	}

	assert_changes!(
		&ScalarNamedVecNotIgnored { some_ints: vec![100] },
		&ScalarNamedVecNotIgnored { some_ints: vec![200] },
		Changed(ScalarNamedVecNotIgnoredMutated { some_ints: vec![VecChange::Changed(0, U8Change(100, 200))] }),
	);
}

#[test]
fn test_struct_1_named_field_comparable_synthetic() {
	#[derive(Comparable)]
	pub struct Synthetics {
		#[comparable_synthetic {
            let full_value = |x: &Self| -> u8 { x.ensemble.iter().sum() };
        }]
		#[comparable_ignore]
		pub ensemble: Vec<u8>,
		pub some_int: u8,
	}

	assert_changes!(
		&Synthetics { ensemble: vec![100], some_int: 100 },
		&Synthetics { ensemble: vec![200], some_int: 100 },
		Changed(vec![SyntheticsChange::FullValue(U8Change(100, 200))]),
	);
}

#[test]
fn test_struct_3_named_fields_varying_visibility() {
	#[derive(Comparable)]
	pub struct Visible {
		pub pub_int_one: i32,
		pub(crate) pub_crate_int_two: i32,
		private_int_three: i32,
	}

	assert_changes!(
		&Visible { pub_int_one: 1, pub_crate_int_two: 2, private_int_three: 3 },
		&Visible { pub_int_one: 4, pub_crate_int_two: 5, private_int_three: 6 },
		Changed(vec![
			VisibleChange::PubIntOne(I32Change(1, 4)),
			VisibleChange::PubCrateIntTwo(I32Change(2, 5)),
			VisibleChange::PrivateIntThree(I32Change(3, 6)),
		]),
	);
}
