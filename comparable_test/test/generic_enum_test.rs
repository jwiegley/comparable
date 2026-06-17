#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(dead_code)]

use comparable::Comparable;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Comparable)]
pub enum IdOrObject<T: PartialEq + Debug> {
	ID(String),
	Object(T),
}

// A generic struct with a single field: its `Change` type is the field's
// change directly (no `Vec`).
#[derive(Debug, PartialEq, Comparable)]
pub struct Single<T: PartialEq + Debug> {
	pub value: T,
}

// A generic struct with multiple fields: its `Change` is a `Vec<PairChange<T>>`.
// Regression test for the derive macro emitting `Vec<PairChange>` (missing the
// `<T>`) in the generated `comparison` body, which failed to compile (E0107).
#[derive(Debug, PartialEq, Comparable)]
pub struct Pair<T: PartialEq + Debug> {
	pub name: String,
	pub value: T,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_generic_enum_basic() {
		let id1 = IdOrObject::<i32>::ID("abc".to_string());
		let id2 = IdOrObject::<i32>::ID("def".to_string());

		let changes = id1.comparison(&id2);
		println!("Changes: {:?}", changes);
	}

	#[test]
	fn test_generic_enum_with_object() {
		let obj1 = IdOrObject::Object(42);
		let obj2 = IdOrObject::Object(100);

		let changes = obj1.comparison(&obj2);
		println!("Changes: {:?}", changes);
	}

	#[test]
	fn test_generic_struct_single_field() {
		use comparable::Changed;
		let a = Single { value: 1i32 };
		let b = Single { value: 1i32 };
		assert!(a.comparison(&b).is_unchanged());

		let c = Single { value: 2i32 };
		assert!(matches!(a.comparison(&c), Changed::Changed(_)));
	}

	#[test]
	fn test_generic_struct_multi_field() {
		use comparable::Changed;

		// Unchanged.
		let a = Pair { name: "x".to_string(), value: 1i32 };
		let b = Pair { name: "x".to_string(), value: 1i32 };
		assert!(a.comparison(&b).is_unchanged());

		// Both fields changed -> exactly two entries, in field order.
		let c = Pair { name: "y".to_string(), value: 2i32 };
		match a.comparison(&c) {
			Changed::Changed(changes) => {
				assert_eq!(
					changes,
					vec![
						PairChange::Name(comparable::StringChange("x".to_string(), "y".to_string())),
						PairChange::Value(comparable::I32Change(1, 2)),
					]
				);
			}
			Changed::Unchanged => panic!("expected a change"),
		}

		// Only one field changed -> exactly one entry.
		let d = Pair { name: "x".to_string(), value: 99i32 };
		match a.comparison(&d) {
			Changed::Changed(changes) => assert_eq!(changes.len(), 1),
			Changed::Unchanged => panic!("expected a change"),
		}
	}
}
