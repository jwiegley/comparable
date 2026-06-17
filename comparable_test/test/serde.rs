//! Regression tests for https://github.com/jwiegley/comparable/issues/12:
//! "Can't serialize changes".
//!
//! With the `serde` feature enabled, every `Comparable::Desc` and
//! `Comparable::Change` type derives `Serialize`/`Deserialize`.  Before the
//! fix, this guarantee was invisible in a *generic* context, because the
//! `Comparable` trait did not bound its associated types with `Serialize`.
//! That broke two things, both reproduced verbatim below:
//!
//!   1. A `#[derive(Serialize)]` type with a `<T as Comparable>::Change` field
//!      (the reporter's `DiffReport`) failed to compile.
//!   2. `#[derive(Comparable)]` on a generic type that *also* derived
//!      `Serialize` failed to compile, because the generated `Desc`/`Change`
//!      types could not prove `<T as Comparable>::Desc: Serialize`.
//!
//! This target is only built when the `serde` feature is on (see
//! `required-features` in `comparable_test/Cargo.toml`).

#![allow(dead_code)]

use comparable::{Changed, Comparable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

// --- Compile-time guarantees ------------------------------------------------
//
// These functions never run; they exist so the build fails if any built-in or
// derived `Desc`/`Change` type stops being serde-serializable.

fn assert_serialize<T: Serialize>() {}
fn assert_deserialize<T: serde::de::DeserializeOwned>() {}

fn assert_change_is_serde<T: Comparable>() {
	assert_serialize::<T::Desc>();
	assert_serialize::<T::Change>();
	assert_deserialize::<T::Desc>();
	assert_deserialize::<T::Change>();
}

#[test]
fn builtin_desc_and_change_are_serde() {
	assert_change_is_serde::<i32>();
	assert_change_is_serde::<bool>();
	assert_change_is_serde::<char>();
	assert_change_is_serde::<f64>();
	assert_change_is_serde::<String>();
	assert_change_is_serde::<Option<u32>>();
	assert_change_is_serde::<Vec<u32>>();
	assert_change_is_serde::<std::collections::BTreeSet<u32>>();
	assert_change_is_serde::<std::collections::HashSet<u32>>();
	assert_change_is_serde::<std::collections::BTreeMap<String, u32>>();
	assert_change_is_serde::<std::collections::HashMap<String, u32>>();
	assert_change_is_serde::<[u32; 4]>();
	assert_change_is_serde::<(u32, String)>();
	assert_change_is_serde::<(u8, u16, u32)>();
	assert_change_is_serde::<Box<u32>>();
	assert_change_is_serde::<std::path::PathBuf>();
	// A generic projection: the bound holds for *any* `T: Comparable`.
	assert_serialize::<<Vec<i32> as Comparable>::Change>();
}

// --- Problem 1: a `#[derive(Serialize)]` report holding `Change` values ------
//
// This is the reporter's struct, reduced to its essence.  It must *compile*.

#[derive(Serialize)]
struct DiffReport<T: Comparable> {
	added: Vec<String>,
	removed: Vec<String>,
	modified: HashMap<String, <T as Comparable>::Change>,
}

#[test]
fn diff_report_with_change_field_serializes() {
	let mut modified = HashMap::new();
	if let Changed::Changed(change) = 100i32.comparison(&200) {
		modified.insert("answer".to_string(), change);
	}
	let report = DiffReport::<i32> { added: vec!["a".to_string()], removed: vec!["b".to_string()], modified };

	let json = serde_json::to_value(&report).expect("DiffReport should serialize");
	assert_eq!(json["added"], serde_json::json!(["a"]));
	assert_eq!(json["removed"], serde_json::json!(["b"]));
	// `I32Change(100, 200)` is a tuple struct → serializes as a two-element array.
	assert_eq!(json["modified"]["answer"], serde_json::json!([100, 200]));
}

// --- Problem 2: deriving `Comparable` and `Serialize` on a generic type ------
//
// This is the reporter's `IdOrObject`, reduced to its essence.  Deriving
// `Comparable` here used to fail with E0277 inside the derive macro.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Comparable)]
pub enum IdOrObject<T: Clone + PartialEq + Debug + Comparable> {
	Id(#[comparable_ignore] u64),
	Object(T),
}

#[test]
fn generic_enum_change_round_trips() {
	let a = IdOrObject::<i32>::Object(1);
	let b = IdOrObject::<i32>::Object(2);

	let change = match a.comparison(&b) {
		Changed::Changed(c) => c,
		Changed::Unchanged => panic!("expected a change"),
	};

	let json = serde_json::to_string(&change).expect("Change should serialize");
	let back: <IdOrObject<i32> as Comparable>::Change = serde_json::from_str(&json).expect("Change should deserialize");
	assert_eq!(change, back);
}

#[test]
fn generic_enum_desc_round_trips() {
	let value = IdOrObject::<i32>::Object(7);
	let desc = value.describe();

	let json = serde_json::to_string(&desc).expect("Desc should serialize");
	let back: <IdOrObject<i32> as Comparable>::Desc = serde_json::from_str(&json).expect("Desc should deserialize");
	assert_eq!(desc, back);
}

// A generic *struct* deriving both, for good measure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Comparable)]
pub struct Wrapper<T: Clone + PartialEq + Debug + Comparable> {
	name: String,
	payload: T,
}

#[test]
fn generic_struct_change_round_trips() {
	let a = Wrapper { name: "x".to_string(), payload: 1u32 };
	let b = Wrapper { name: "y".to_string(), payload: 2u32 };

	let change = match a.comparison(&b) {
		Changed::Changed(c) => c,
		Changed::Unchanged => panic!("expected a change"),
	};

	let json = serde_json::to_string(&change).expect("Change should serialize");
	let back: <Wrapper<u32> as Comparable>::Change = serde_json::from_str(&json).expect("Change should deserialize");
	assert_eq!(change, back);
}
