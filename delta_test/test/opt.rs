use delta::{assert_changes, Changed, EnumChange, UsizeChange};

#[test]
fn test_option() {
    assert_changes(
        &(None as Option<usize>),
        &(None as Option<usize>),
        Changed::Unchanged,
    );
    assert_changes(
        &None,
        &Some(100 as usize),
        Changed::Changed(EnumChange::DiffVariant(None, Some(100))),
    );
    assert_changes(
        &Some(100 as usize),
        &None,
        Changed::Changed(EnumChange::DiffVariant(Some(100), None)),
    );
    assert_changes(&Some(100 as usize), &Some(100 as usize), Changed::Unchanged);
    assert_changes(
        &Some(100 as usize),
        &Some(200 as usize),
        Changed::Changed(EnumChange::SameVariant(UsizeChange(100, 200))),
    );
}
