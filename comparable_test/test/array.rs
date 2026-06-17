use comparable::{Changed::*, *};

// Regression test for https://github.com/jwiegley/comparable/issues/13:
// arrays lost their `Comparable` impl in 0.5.6 because `pub mod array;` was
// dropped from the generated `lib.rs`.  These tests exercise the array impl so
// the module stays wired in.

// The exact reproduction from the issue report.
#[test]
fn test_array_issue_13_repro() {
	let t1 = [0u8; 5];
	let t2 = [0u8; 5];
	assert_changes!(&t1, &t2, Unchanged);
	assert_eq!(t1.comparison(&t2), Unchanged);
}

#[test]
fn test_array_unchanged() {
	let t1 = [1u8, 2, 3, 4, 5];
	let t2 = [1u8, 2, 3, 4, 5];
	assert_changes!(&t1, &t2, Unchanged);
}

#[test]
fn test_array_single_change() {
	let t1 = [1u8, 2, 3, 4, 5];
	let t2 = [1u8, 9, 3, 4, 5];
	assert_changes!(&t1, &t2, Changed([Unchanged, Changed(U8Change(2, 9)), Unchanged, Unchanged, Unchanged]));
}

#[test]
fn test_array_multiple_changes() {
	let t1 = [1u32, 2, 3];
	let t2 = [4u32, 2, 6];
	assert_changes!(&t1, &t2, Changed([Changed(U32Change(1, 4)), Unchanged, Changed(U32Change(3, 6))]));
}

#[test]
fn test_array_describe() {
	let t = [10u8, 20, 30];
	assert_eq!(t.describe(), [10u8, 20, 30]);
}
