// use serde;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use crate::types::{Changed, Comparable};

#[derive(
    PartialEq,
    Debug, // , serde::Serialize, serde::Deserialize
)]
pub enum MapChange<Key, Desc, Change> {
    Added(Key, Desc),
    Changed(Key, Change),
    Removed(Key),
}

impl<Key: Ord + Clone + Debug, Value: Comparable> Comparable for BTreeMap<Key, Value> {
    type Desc = BTreeMap<Key, Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter()
            .map(|(k, v)| (k.clone(), v.describe()))
            .collect()
    }

    type Change = Vec<MapChange<Key, Value::Desc, Value::Change>>;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        let mut changes = Vec::new();
        changes.append(
            &mut other
                .iter()
                .map(|(k, v)| {
                    if let Some(vo) = self.get(k) {
                        vo.comparison(v)
                            .map(|changes| MapChange::Changed(k.clone(), changes))
                    } else {
                        Changed::Changed(MapChange::Added(k.clone(), v.describe()))
                    }
                })
                .flatten()
                .collect(),
        );
        changes.append(
            &mut self
                .iter()
                .map(|(k, _v)| {
                    if !other.contains_key(k) {
                        Changed::Changed(MapChange::Removed(k.clone()))
                    } else {
                        Changed::Unchanged
                    }
                })
                .flatten()
                .collect(),
        );
        if changes.is_empty() {
            Changed::Unchanged
        } else {
            Changed::Changed(changes)
        }
    }
}

fn to_btreemap<K: Clone + Ord, V>(map: &HashMap<K, V>) -> BTreeMap<K, &V> {
    map.iter()
        .map(|(k, v)| (k.clone(), v))
        .collect::<BTreeMap<K, &V>>()
        .into_iter()
        .collect()
}

impl<Key: Ord + Clone + Debug, Value: Comparable> Comparable for HashMap<Key, Value> {
    type Desc = BTreeMap<Key, Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter()
            .map(|(k, v)| (k.clone(), v.describe()))
            .collect()
    }

    type Change = Vec<MapChange<Key, Value::Desc, Value::Change>>;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        to_btreemap(self).comparison(&to_btreemap(other))
    }
}
