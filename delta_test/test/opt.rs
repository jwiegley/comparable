use delta::{assert_changes, Changed, EnumChange, I32Change};

#[test]
fn test_option() {
    assert_changes(
        &(None as Option<i32>),
        &(None as Option<i32>),
        Changed::Unchanged,
    );
    assert_changes(
        &None,
        &Some(100),
        Changed::Changed(EnumChange::DiffVariant(None, Some(100))),
    );
    assert_changes(
        &Some(100),
        &None,
        Changed::Changed(EnumChange::DiffVariant(Some(100), None)),
    );
    assert_changes(&Some(100), &Some(100), Changed::Unchanged);
    assert_changes(
        &Some(100),
        &Some(200),
        Changed::Changed(EnumChange::SameVariant(I32Change(100, 200))),
    );
}
