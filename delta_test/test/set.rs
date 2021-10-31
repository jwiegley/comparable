#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

use std::collections::{BTreeSet, HashSet};

use delta::{assert_changes, Changed, I32Change, SetChange, VecChange};

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
            VecChange::Changed(1, I32Change(3, 2)),
            VecChange::Added(2, 3),
        ]),
    );
    assert_changes(
        &vec![1 as i32, 2, 3],
        &vec![1 as i32, 3],
        Changed::Changed(vec![
            VecChange::Changed(1, I32Change(2, 3)),
            VecChange::Removed(2, 3),
        ]),
    );
    assert_changes(
        &vec![1 as i32, 2, 3],
        &vec![1 as i32, 4, 3],
        Changed::Changed(vec![VecChange::Changed(1, I32Change(2, 4))]),
    );
}

// jww (2021-10-31): Need to sort hashsets before comparison
#[test]
fn test_hashset() {
    assert_changes(&(vec![] as Vec<i32>), &vec![], Changed::Unchanged);
    assert_changes(
        &HashSet::from(vec![].into_iter().collect()),
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        Changed::Changed(vec![
            SetChange::Added(1),
            SetChange::Added(2),
            SetChange::Added(3),
        ]),
    );
    assert_changes(
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &HashSet::from(vec![].into_iter().collect()),
        Changed::Changed(vec![
            SetChange::Removed(1),
            SetChange::Removed(2),
            SetChange::Removed(3),
        ]),
    );
    assert_changes(
        &HashSet::from(vec![1 as i32, 2].into_iter().collect()),
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(3)]),
    );
    assert_changes(
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &HashSet::from(vec![1 as i32, 2].into_iter().collect()),
        Changed::Changed(vec![SetChange::Removed(3)]),
    );
    assert_changes(
        &HashSet::from(vec![1 as i32, 3].into_iter().collect()),
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(2)]),
    );
    assert_changes(
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &HashSet::from(vec![1 as i32, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Removed(2)]),
    );
    assert_changes(
        &HashSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &HashSet::from(vec![1 as i32, 4, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(4), SetChange::Removed(2)]),
    );
}

#[test]
fn test_btreeset() {
    assert_changes(&(vec![] as Vec<i32>), &vec![], Changed::Unchanged);
    assert_changes(
        &BTreeSet::from(vec![].into_iter().collect()),
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        Changed::Changed(vec![
            SetChange::Added(1),
            SetChange::Added(2),
            SetChange::Added(3),
        ]),
    );
    assert_changes(
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &BTreeSet::from(vec![].into_iter().collect()),
        Changed::Changed(vec![
            SetChange::Removed(1),
            SetChange::Removed(2),
            SetChange::Removed(3),
        ]),
    );
    assert_changes(
        &BTreeSet::from(vec![1 as i32, 2].into_iter().collect()),
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(3)]),
    );
    assert_changes(
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &BTreeSet::from(vec![1 as i32, 2].into_iter().collect()),
        Changed::Changed(vec![SetChange::Removed(3)]),
    );
    assert_changes(
        &BTreeSet::from(vec![1 as i32, 3].into_iter().collect()),
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(2)]),
    );
    assert_changes(
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &BTreeSet::from(vec![1 as i32, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Removed(2)]),
    );
    assert_changes(
        &BTreeSet::from(vec![1 as i32, 2, 3].into_iter().collect()),
        &BTreeSet::from(vec![1 as i32, 4, 3].into_iter().collect()),
        Changed::Changed(vec![SetChange::Added(4), SetChange::Removed(2)]),
    );
}
