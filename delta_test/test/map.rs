use std::collections::{BTreeMap, HashMap};

use delta::{assert_changes, Changed, I32Change, MapChange};

#[test]
fn test_hashmap() {
    assert_changes(
        &HashMap::<i32, i32>::new(),
        &HashMap::<i32, i32>::new(),
        Changed::Unchanged,
    );
    assert_changes(
        &HashMap::new(),
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![
            MapChange::Added(0, 1),
            MapChange::Added(1, 2),
            MapChange::Added(2, 3),
        ]),
    );
    assert_changes(
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &HashMap::new(),
        Changed::Changed(vec![
            MapChange::Removed(0),
            MapChange::Removed(1),
            MapChange::Removed(2),
        ]),
    );
    assert_changes(
        &HashMap::from(vec![(0, 1 as i32), (1, 2)].into_iter().collect()),
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Added(2, 3)]),
    );
    assert_changes(
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &HashMap::from(vec![(0, 1 as i32), (1, 2)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Removed(2)]),
    );
    assert_changes(
        &HashMap::from(vec![(0, 1 as i32), (2, 3)].into_iter().collect()),
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Added(1, 2)]),
    );
    assert_changes(
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &HashMap::from(vec![(0, 1 as i32), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Removed(1)]),
    );
    assert_changes(
        &HashMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &HashMap::from(vec![(0, 1 as i32), (1, 4), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Changed(1, I32Change(2, 4))]),
    );
}

#[test]
fn test_btreemap() {
    assert_changes(
        &BTreeMap::<i32, i32>::new(),
        &BTreeMap::<i32, i32>::new(),
        Changed::Unchanged,
    );
    assert_changes(
        &BTreeMap::new(),
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![
            MapChange::Added(0, 1),
            MapChange::Added(1, 2),
            MapChange::Added(2, 3),
        ]),
    );
    assert_changes(
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &BTreeMap::new(),
        Changed::Changed(vec![
            MapChange::Removed(0),
            MapChange::Removed(1),
            MapChange::Removed(2),
        ]),
    );
    assert_changes(
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2)].into_iter().collect()),
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Added(2, 3)]),
    );
    assert_changes(
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Removed(2)]),
    );
    assert_changes(
        &BTreeMap::from(vec![(0, 1 as i32), (2, 3)].into_iter().collect()),
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Added(1, 2)]),
    );
    assert_changes(
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &BTreeMap::from(vec![(0, 1 as i32), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Removed(1)]),
    );
    assert_changes(
        &BTreeMap::from(vec![(0, 1 as i32), (1, 2), (2, 3)].into_iter().collect()),
        &BTreeMap::from(vec![(0, 1 as i32), (1, 4), (2, 3)].into_iter().collect()),
        Changed::Changed(vec![MapChange::Changed(1, I32Change(2, 4))]),
    );
}
