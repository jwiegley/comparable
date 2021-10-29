use serde;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

use crate::types::Delta;

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum MapChange<Key, Desc, Change> {
    Added(Key, Desc),
    Changed(Key, Change),
    Removed(Key),
}

impl<Key: Ord + Clone + Debug, Value: Delta> Delta for BTreeMap<Key, Value> {
    type Desc = BTreeMap<Key, Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter()
            .map(|(k, v)| (k.clone(), v.describe()))
            .collect()
    }

    type Change = Vec<MapChange<Key, Value::Desc, Value::Change>>;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        let mut changes = Vec::new();
        changes.append(
            &mut other
                .iter()
                .map(|(k, v)| {
                    if let Some(vo) = self.get(k) {
                        vo.delta(v)
                            .map(|changes| MapChange::Changed(k.clone(), changes))
                    } else {
                        Some(MapChange::Added(k.clone(), v.describe()))
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
                        Some(MapChange::Removed(k.clone()))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect(),
        );
        if changes.is_empty() {
            None
        } else {
            Some(changes)
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

impl<Key: Ord + Clone + Debug, Value: Delta> Delta for HashMap<Key, Value> {
    type Desc = BTreeMap<Key, Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter()
            .map(|(k, v)| (k.clone(), v.describe()))
            .collect()
    }

    type Change = Vec<MapChange<Key, Value::Desc, Value::Change>>;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        to_btreemap(self).delta(&to_btreemap(other))
    }
}
