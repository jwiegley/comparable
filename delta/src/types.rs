use std::fmt::Debug;

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
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

impl<T> Into<Option<T>> for Changed<T> {
    #[inline]
    fn into(self) -> Option<T> {
        match self {
            Changed::Unchanged => None,
            Changed::Changed(x) => Some(x),
        }
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

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub enum EnumChange<Desc, Change> {
    SameVariant(Change),
    DiffVariant(Desc, Desc),
}

pub trait Delta {
    /// A type that describes the type under consideration. For many types
    /// this is just the type itself, but some large structures are better
    /// described by a Delta from the Default, for exmaple.
    type Desc: PartialEq + Debug;

    fn describe(&self) -> Self::Desc;

    /// A type that describes the changes between to values of a type.
    type Change: PartialEq + Debug;

    fn delta(&self, other: &Self) -> Changed<Self::Change>;
}

impl<T: Delta> Delta for &T {
    type Desc = T::Desc;

    fn describe(&self) -> Self::Desc {
        (*self).describe()
    }

    type Change = T::Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        (*self).delta(other)
    }
}
