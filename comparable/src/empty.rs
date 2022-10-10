use crate::types::{Changed, Comparable};

impl<T> Comparable for std::iter::Empty<T> {
	type Desc = ();

	fn describe(&self) -> Self::Desc {}

	type Change = ();

	fn comparison(&self, _other: &Self) -> Changed<Self::Change> {
		Changed::Unchanged
	}
}
