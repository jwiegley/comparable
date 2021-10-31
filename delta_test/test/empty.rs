use delta::{assert_changes, Changed};

#[test]
fn test_empty() {
    assert_changes(
        &std::iter::empty::<()>(),
        &std::iter::empty::<()>(),
        Changed::Unchanged,
    );
}
