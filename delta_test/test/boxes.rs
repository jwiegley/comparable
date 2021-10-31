use delta::{assert_changes, Changed, I32Change};

#[test]
fn test_box() {
    assert_changes(&Box::new(100), &Box::new(100), Changed::Unchanged);
    assert_changes(
        &Box::new(100),
        &Box::new(200),
        Changed::Changed(I32Change(100, 200)),
    );
}
