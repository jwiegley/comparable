pub mod map;
pub mod opt;
pub mod scalar;
pub mod types;
pub mod vec;

pub use crate::map::*;
pub use crate::opt::*;
pub use crate::scalar::*;
pub use crate::types::*;
pub use crate::vec::*;

pub fn assert_changes<T: Delta>(left: &T, right: &T, expected: Option<<T as Delta>::Change>) {
    pretty_assertions::assert_eq!(expected, left.delta(right))
}
