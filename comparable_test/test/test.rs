#![allow(clippy::derive_partial_eq_without_eq)]
// Many fixtures below carry deliberately-unused fields to exercise
// #[comparable_ignore] and the various derive shapes.
#![allow(dead_code)]

mod boxes;
mod empty;
mod enums;
mod map;
mod opt;
mod scalar;
mod set;
mod string;
mod structs;
mod unions;
