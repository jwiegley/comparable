use delta::{assert_changes, Changed, StringChange};

#[test]
fn test_string() {
    assert_changes(
        &("hello".to_string()),
        &("hello".to_string()),
        Changed::Unchanged,
    );
    assert_changes(
        &("hello".to_string()),
        &("goodbye".to_string()),
        Changed::Changed(StringChange("hello".to_string(), "goodbye".to_string())),
    );
}
