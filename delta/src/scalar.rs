// use serde;

use crate::types::{Changed, Delta};

impl Delta for () {
    type Desc = ();

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = ();

    fn delta(&self, _other: &Self) -> Changed<Self::Change> {
        Changed::Unchanged
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub struct BoolChange(pub bool, pub bool);

impl Delta for bool {
    type Desc = bool;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = BoolChange;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(BoolChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct U8Change(pub u8, pub u8);

impl Delta for u8 {
    type Desc = u8;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U8Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U8Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct I8Change(pub i8, pub i8);

impl Delta for i8 {
    type Desc = i8;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I8Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I8Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct U16Change(pub u16, pub u16);

impl Delta for u16 {
    type Desc = u16;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U16Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U16Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct I16Change(pub i16, pub i16);

impl Delta for i16 {
    type Desc = i16;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I16Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I16Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct U32Change(pub u32, pub u32);

impl Delta for u32 {
    type Desc = u32;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U32Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U32Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct I32Change(pub i32, pub i32);

impl Delta for i32 {
    type Desc = i32;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I32Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I32Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct U64Change(pub u64, pub u64);

impl Delta for u64 {
    type Desc = u64;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = U64Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(U64Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct I64Change(pub i64, pub i64);

impl Delta for i64 {
    type Desc = i64;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = I64Change;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(I64Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct UsizeChange(pub usize, pub usize);

impl Delta for usize {
    type Desc = usize;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = UsizeChange;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(UsizeChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct IsizeChange(pub isize, pub isize);

impl Delta for isize {
    type Desc = isize;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = IsizeChange;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(IsizeChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct F32Change(pub f32, pub f32);

impl Delta for f32 {
    type Desc = f32;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = F32Change;

    #[allow(clippy::float_cmp)]
    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(F32Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct F64Change(pub f64, pub f64);

impl Delta for f64 {
    type Desc = f64;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = F64Change;

    #[allow(clippy::float_cmp)]
    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(F64Change(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct CharChange(pub char, pub char);

impl Delta for char {
    type Desc = char;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = CharChange;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(CharChange(*self, *other))
        } else {
            Changed::Unchanged
        }
    }
}

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]

pub struct StringChange(pub String, pub String);

impl Delta for String {
    type Desc = String;

    fn describe(&self) -> Self::Desc {
        self.to_string()
    }

    type Change = StringChange;

    fn delta(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(StringChange(self.to_string(), other.to_string()))
        } else {
            Changed::Unchanged
        }
    }
}
