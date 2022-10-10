use comparable::{
	assert_changes, pretty_assert_changes, prop_assert_changes, prop_pretty_assert_changes, Changed::*, I32Change,
};
use proptest::prelude::*;

#[test]
fn test_box() {
	assert_changes!(&Box::new(100), &Box::new(200), Changed(I32Change(100, 200)));
	pretty_assert_changes!(&Box::new(100), &Box::new(200), Changed(I32Change(100, 200)));
}

proptest! {

#[test]
fn test_box_proptest(value in 100i32..=100) {
	prop_assert_changes!(&Box::new(value), &Box::new(value), Unchanged);
	prop_pretty_assert_changes!(&Box::new(value), &Box::new(value), Unchanged::<I32Change>);
}

}
