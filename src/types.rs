use std::fmt::Debug;

pub trait Delta {
    /// A type that describes the type under consideration. For many types
    /// this is just the type itself, but some large structures are better
    /// described by a Delta from the Default, for exmaple.
    type Desc: PartialEq + Debug;

    fn describe(&self) -> Self::Desc;

    /// A type that describes the changes between to values of a type.
    type Change: PartialEq + Debug;

    fn delta(&self, other: &Self) -> Option<Self::Change>;
}

impl<T: Delta> Delta for &T {
    type Desc = T::Desc;

    fn describe(&self) -> Self::Desc {
        (*self).describe()
    }

    type Change = T::Change;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        (*self).delta(other)
    }
}

pub fn assert_changes<T: Delta>(left: &T, right: &T, expected: Option<<T as Delta>::Change>) {
    pretty_assertions::assert_eq!(expected, left.delta(right))
}
