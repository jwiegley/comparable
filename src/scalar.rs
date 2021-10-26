use crate::types::Delta;

#[derive(PartialEq, Debug)]
pub struct BoolChange(bool, bool);

impl Delta for bool {
    type Desc = bool;

    fn describe(&self) -> Self::Desc {
        *self
    }

    type Change = BoolChange;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        if self != other {
            Some(BoolChange(*self, *other))
        } else {
            None
        }
    }
}

// u8
// i8
// u16
// i16
// u32
// i32
// u64
// i64
// usize
// isize
// f32
// f64
// char
// String
