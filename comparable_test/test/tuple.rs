use comparable::{Changed::*, *};

// Regression test for https://github.com/jwiegley/comparable/issues/13:
// tuples lost their `Comparable` impl in 0.5.6 because `pub mod tuple;` was
// dropped from the generated `lib.rs`.

#[test]
fn test_tuple_pair_unchanged() {
	assert_changes!(&(1u8, 2u32), &(1u8, 2u32), Unchanged);
}

#[test]
fn test_tuple_pair_changed() {
	assert_changes!(&(1u8, 2u32), &(1u8, 9u32), Changed((Unchanged, Changed(U32Change(2, 9)))));
}

#[test]
fn test_tuple_pair_both_changed() {
	assert_changes!(&(1u8, 2u32), &(4u8, 9u32), Changed((Changed(U8Change(1, 4)), Changed(U32Change(2, 9)))));
}

#[test]
fn test_tuple_triple_changed() {
	assert_changes!(&(1u8, 2u32, 3u64), &(1u8, 9u32, 3u64), Changed((Unchanged, Changed(U32Change(2, 9)), Unchanged)));
}

#[test]
fn test_tuple_describe() {
	let t = (1u8, 2u32);
	assert_eq!(t.describe(), (1u8, 2u32));
}
