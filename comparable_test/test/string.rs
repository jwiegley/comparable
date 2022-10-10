use comparable::{assert_changes, Changed::*, StringChange};

#[test]
fn test_string() {
	assert_changes!(&("hello".to_string()), &("hello".to_string()), Unchanged);
	assert_changes!(
		&("hello".to_string()),
		&("goodbye".to_string()),
		Changed(StringChange("hello".to_string(), "goodbye".to_string())),
	);
}
