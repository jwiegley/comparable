#![no_main]
#![allow(clippy::derive_partial_eq_without_eq)]

use arbitrary::Arbitrary;
use comparable::Comparable;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Clone, Debug, comparable::Comparable)]
struct Inner {
    a: i64,
    b: Vec<u8>,
    c: bool,
}

#[derive(Arbitrary, Clone, Debug, comparable::Comparable)]
struct Sample {
    id: u64,
    name: String,
    tags: Vec<i32>,
    nested: Option<Box<Inner>>,
}

fuzz_target!(|pair: (Sample, Sample)| {
    let (x, y) = pair;

    // Invariant: a value compared with itself never reports a change.
    assert!(x.comparison(&x).is_unchanged());
    assert!(y.comparison(&y).is_unchanged());

    // describe() and comparison() must never panic on arbitrary input.
    let _ = x.describe();
    let _ = y.describe();
    let _ = x.comparison(&y);
});
