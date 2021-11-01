pub mod boxes;
pub mod empty;
pub mod map;
pub mod opt;
pub mod scalar;
pub mod set;
pub mod string;
pub mod types;

pub use crate::boxes::*;
pub use crate::map::*;
pub use crate::opt::*;
pub use crate::scalar::*;
pub use crate::set::*;
pub use crate::string::*;
pub use crate::types::*;

pub fn assert_changes<T: Comparable>(left: &T, right: &T, expected: Changed<<T as Comparable>::Change>) {
    pretty_assertions::assert_eq!(expected, left.comparison(right))
}

// Re-export #[derive(Comparable)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "comparable_derive")]
#[doc(hidden)]
pub use comparable_derive::*;
