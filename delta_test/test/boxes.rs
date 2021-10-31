use delta::{assert_changes, Changed, UsizeChange};

#[test]
fn test_box() {
    assert_changes(
        &Box::new(100 as usize),
        &Box::new(100 as usize),
        Changed::Unchanged,
    );
    assert_changes(
        &Box::new(100 as usize),
        &Box::new(200 as usize),
        Changed::Changed(UsizeChange(100, 200)),
    );
}
