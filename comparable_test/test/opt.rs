use comparable::{assert_changes, Changed, I32Change, OptionChange};

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
        Changed::Changed(OptionChange::Different(None, Some(100))),
    );
    assert_changes(
        &Some(100),
        &None,
        Changed::Changed(OptionChange::Different(Some(100), None)),
    );
    assert_changes(&Some(100), &Some(100), Changed::Unchanged);
    assert_changes(
        &Some(100),
        &Some(200),
        Changed::Changed(OptionChange::BothSome(I32Change(100, 200))),
    );
}
