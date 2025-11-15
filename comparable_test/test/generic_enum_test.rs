use comparable::Comparable;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Comparable)]
pub enum IdOrObject<T: PartialEq + Debug> {
	ID(String),
	Object(T),
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
}
