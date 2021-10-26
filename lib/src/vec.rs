use crate::types::Delta;

#[derive(PartialEq, Debug)]
pub enum VecChange<Desc> {
    Added(Desc),
    Removed(Desc),
}

impl<Value: PartialEq + Delta> Delta for Vec<Value> {
    type Desc = Vec<Value::Desc>;

    fn describe(&self) -> Self::Desc {
        self.iter().map(|x| x.describe()).collect()
    }

    type Change = Vec<VecChange<Value::Desc>>;

    fn delta(&self, other: &Self) -> Option<Self::Change> {
        let mut changes = Vec::new();
        changes.append(
            &mut other
                .iter()
                .map(|v| {
                    if self.contains(v) {
                        None
                    } else {
                        Some(VecChange::Added(v.describe()))
                    }
                })
                .flatten()
                .collect(),
        );
        changes.append(
            &mut self
                .iter()
                .map(|v| {
                    if !other.contains(v) {
                        Some(VecChange::Removed(v.describe()))
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
