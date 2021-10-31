use delta::{assert_changes, Changed, I32Change, VecChange};

#[test]
fn test_vec() {
    assert_changes(&(vec![] as Vec<i32>), &vec![], Changed::Unchanged);
    assert_changes(
        &vec![],
        &vec![1 as i32, 2, 3],
        Changed::Changed(vec![
            VecChange::Added(0, 1),
            VecChange::Added(1, 2),
            VecChange::Added(2, 3),
        ]),
    );
    assert_changes(
        &vec![1 as i32, 2, 3],
        &vec![],
        Changed::Changed(vec![
            VecChange::Removed(0, 1),
            VecChange::Removed(1, 2),
            VecChange::Removed(2, 3),
        ]),
    );
    assert_changes(
        &vec![1 as i32, 2],
        &vec![1 as i32, 2, 3],
        Changed::Changed(vec![VecChange::Added(2, 3)]),
    );
    assert_changes(
        &vec![1 as i32, 2, 3],
        &vec![1 as i32, 2],
        Changed::Changed(vec![VecChange::Removed(2, 3)]),
    );
    assert_changes(
        &vec![1 as i32, 3],
        &vec![1 as i32, 2, 3],
        Changed::Changed(vec![
            VecChange::Change(1, I32Change(3, 2)),
            VecChange::Added(2, 3),
        ]),
    );
    assert_changes(
        &vec![1 as i32, 2, 3],
        &vec![1 as i32, 3],
        Changed::Changed(vec![
            VecChange::Change(1, I32Change(2, 3)),
            VecChange::Removed(2, 3),
        ]),
    );
    assert_changes(
        &vec![1 as i32, 2, 3],
        &vec![1 as i32, 4, 3],
        Changed::Changed(vec![VecChange::Change(1, I32Change(2, 4))]),
    );
}

#[test]
fn test_hashset() {}

#[test]
fn test_btreeset() {}
