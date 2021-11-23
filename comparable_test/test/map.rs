#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

use std::collections::{BTreeMap, HashMap};

use comparable::{assert_changes, Changed::*, I32Change, MapChange};

pub struct HashMapBuilder {
    elements: Vec<(i32, i32)>,
}

impl HashMapBuilder {
    fn new() -> Self {
        HashMapBuilder {
            elements: Vec::new(),
        }
    }

    fn add_element(mut self, key: i32, value: i32) -> Self {
        self.elements.push((key, value));
        self
    }

    fn create(self) -> HashMap<i32, i32> {
        self.elements.into_iter().collect()
    }
}

#[test]
fn test_hashmap_example_old() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 100);
    map.insert(2, 200);

    map.remove(&2);

    // But what if `map.remove` had other side-effects? We wouldn't know.
    assert_eq!(map.get(&2), None);
}

#[test]
fn test_hashmap_example_new() {
    let mut map = HashMapBuilder::new()
        .add_element(1, 100)
        .add_element(2, 200)
        .create();
    let initial_map = map.clone();

    map.remove(&2);

    // We assert here that map.remove can only have had one effect.
    assert_changes!(&initial_map, &map, Changed(vec![MapChange::Removed(2)]));
}

#[test]
fn test_hashmap() {
    assert_changes!(
        &HashMap::<i32, i32>::new(),
        &HashMap::<i32, i32>::new(),
        Unchanged,
    );
    assert_changes!(
        &HashMap::new(),
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        Changed(vec![
            MapChange::Added(0, 1),
            MapChange::Added(1, 2),
            MapChange::Added(2, 3),
        ]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        &HashMap::new(),
        Changed(vec![
            MapChange::Removed(0),
            MapChange::Removed(1),
            MapChange::Removed(2),
        ]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        Changed(vec![MapChange::Added(2, 3)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 2)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        Changed(vec![MapChange::Removed(2)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        Changed(vec![MapChange::Added(1, 2)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        &vec![(0, 1 as i32), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        Changed(vec![MapChange::Removed(1)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 4), (2, 3)]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        Changed(vec![MapChange::Changed(1, I32Change(2, 4))]),
    );
}

#[test]
fn test_btreemap() {
    assert_changes!(
        &BTreeMap::<i32, i32>::new(),
        &BTreeMap::<i32, i32>::new(),
        Unchanged,
    );
    assert_changes!(
        &BTreeMap::new(),
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        Changed(vec![
            MapChange::Added(0, 1),
            MapChange::Added(1, 2),
            MapChange::Added(2, 3),
        ]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        &BTreeMap::new(),
        Changed(vec![
            MapChange::Removed(0),
            MapChange::Removed(1),
            MapChange::Removed(2),
        ]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        Changed(vec![MapChange::Added(2, 3)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 2)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        Changed(vec![MapChange::Removed(2)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        Changed(vec![MapChange::Added(1, 2)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        &vec![(0, 1 as i32), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        Changed(vec![MapChange::Removed(1)]),
    );
    assert_changes!(
        &vec![(0, 1 as i32), (1, 2), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        &vec![(0, 1 as i32), (1, 4), (2, 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        Changed(vec![MapChange::Changed(1, I32Change(2, 4))]),
    );
}
