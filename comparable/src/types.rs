use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug)]
pub enum Changed<T> {
	Unchanged,
	Changed(T),
}

impl<T> Changed<T> {
	#[inline]
	pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Changed<U> {
		match self {
			Changed::Unchanged => Changed::Unchanged,
			Changed::Changed(x) => Changed::Changed(f(x)),
		}
	}

	pub fn take(&mut self) -> Option<T> {
		match std::mem::take(self) {
			Changed::Unchanged => None,
			Changed::Changed(x) => Some(x),
		}
	}

	pub fn to_changes(&mut self) -> Vec<T> {
		match std::mem::take(self) {
			Changed::Unchanged => vec![],
			Changed::Changed(x) => vec![x],
		}
	}

	pub fn is_unchanged(&self) -> bool {
		match self {
			Changed::Unchanged => true,
			Changed::Changed(_) => false,
		}
	}
}

impl<T: Default> Changed<T> {
	#[inline]
	pub fn unwrap_or_default(self) -> T {
		match self {
			Changed::Changed(x) => x,
			Changed::Unchanged => Default::default(),
		}
	}
}

impl<T> Default for Changed<T> {
	#[inline]
	fn default() -> Self {
		Changed::Unchanged
	}
}

impl<T> From<Option<T>> for Changed<T> {
	#[inline]
	fn from(opt: Option<T>) -> Self {
		match opt {
			None => Changed::Unchanged,
			Some(x) => Changed::Changed(x),
		}
	}
}

impl<T> Iterator for Changed<T> {
	type Item = T;
	fn next(&mut self) -> Option<T> {
		self.take()
	}
}

/// A bound shared by every [`Comparable::Desc`] and [`Comparable::Change`]
/// type.
///
/// When the `serde` feature is enabled this requires `Serialize` and
/// `DeserializeOwned`, so that descriptions and change sets can be serialized
/// even through a generic `T: Comparable` (where the concrete `Desc`/`Change`
/// types — and therefore their derived serde impls — are not yet known). When
/// the feature is disabled it is an empty, blanket-implemented marker that adds
/// no constraint.
///
/// This is an implementation detail of the trait's bounds and is not meant to
/// be named or implemented directly.
#[cfg(feature = "serde")]
#[doc(hidden)]
pub trait MaybeSerde: serde::Serialize + serde::de::DeserializeOwned {}
#[cfg(feature = "serde")]
impl<T: serde::Serialize + serde::de::DeserializeOwned> MaybeSerde for T {}

#[cfg(not(feature = "serde"))]
#[doc(hidden)]
pub trait MaybeSerde {}
#[cfg(not(feature = "serde"))]
impl<T> MaybeSerde for T {}

pub trait Comparable {
	/// Describes the type under consideration. For types that use
	/// `#[derive(Comparable)]` this is a mirror of the type itself, where all
	/// field types refer to the `Comparable::Desc` associated type of the
	/// original type.
	///
	/// With the `serde` feature enabled this type is always `Serialize` and
	/// `DeserializeOwned`.
	type Desc: PartialEq + Debug + MaybeSerde;

	/// Describe a value of a type.
	fn describe(&self) -> Self::Desc;

	/// Reflects all changes between two values of a type. The exact nature of
	/// this type depends on the type being compared, for example, singleton
	/// struts vary from structs with multiple fields. Please see the [full
	/// documentation](https://docs.rs/comparable) for more details.
	///
	/// With the `serde` feature enabled this type is always `Serialize` and
	/// `DeserializeOwned`.
	type Change: PartialEq + Debug + MaybeSerde;

	/// Compare two values of a type, reporting whether they differ and what
	/// the complete set of differences looks like. This is used by the
	/// `comparable::assert_changes` function so that tests can ensure that
	/// what was expected to happen did happen -- and nothing more.
	fn comparison(&self, other: &Self) -> Changed<Self::Change>;
}

impl<T: Comparable> Comparable for &T {
	type Desc = T::Desc;

	fn describe(&self) -> Self::Desc {
		(*self).describe()
	}

	type Change = T::Change;

	fn comparison(&self, other: &Self) -> Changed<Self::Change> {
		(*self).comparison(other)
	}
}
