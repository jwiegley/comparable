// use serde;

use crate::types::{Changed, Comparable};

impl Comparable for () {
    type Desc = ();

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = ();

    fn comparison(&self, _other: &Self) -> Changed<Self::Change> {
        Changed::Unchanged
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct BoolChange(pub bool, pub bool);

impl Comparable for bool {
    type Desc = bool;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = BoolChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(BoolChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct U8Change(pub u8, pub u8);

impl Comparable for u8 {
    type Desc = u8;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U8Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U8Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct I8Change(pub i8, pub i8);

impl Comparable for i8 {
    type Desc = i8;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I8Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I8Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]

pub struct U16Change(pub u16, pub u16);

impl Comparable for u16 {
    type Desc = u16;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U16Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U16Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct I16Change(pub i16, pub i16);

impl Comparable for i16 {
    type Desc = i16;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I16Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I16Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct U32Change(pub u32, pub u32);

impl Comparable for u32 {
    type Desc = u32;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U32Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U32Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct I32Change(pub i32, pub i32);

impl Comparable for i32 {
    type Desc = i32;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I32Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I32Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct U64Change(pub u64, pub u64);

impl Comparable for u64 {
    type Desc = u64;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U64Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U64Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]

pub struct I64Change(pub i64, pub i64);

impl Comparable for i64 {
    type Desc = i64;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I64Change;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I64Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct UsizeChange(pub usize, pub usize);

impl Comparable for usize {
    type Desc = usize;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = UsizeChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(UsizeChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct IsizeChange(pub isize, pub isize);

impl Comparable for isize {
    type Desc = isize;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = IsizeChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(IsizeChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug)]
pub struct F32Change(pub f32, pub f32);

impl Comparable for f32 {
    type Desc = f32;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = F32Change;

    #[allow(clippy::float_cmp)]
    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(F32Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Debug)]
pub struct F64Change(pub f64, pub f64);

impl Comparable for f64 {
    type Desc = f64;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = F64Change;

    #[allow(clippy::float_cmp)]
    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(F64Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Debug)]
pub struct CharChange(pub char, pub char);

impl Comparable for char {
    type Desc = char;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = CharChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(CharChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}
