use crate::types::{Changed, Comparable};
use std::convert::TryInto;

fn convert_vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

impl<T: Comparable, const N: usize> Comparable for [T; N] {
    type Desc = [T::Desc; N];

    fn describe(&self) -> Self::Desc {
        let v = self.iter().map(|v| v.describe()).collect::<Vec<_>>();
        convert_vec_to_array(v)
    }

    type Change = [Changed<T::Change>; N];

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        let mut result: Self::Change = [(); N].map(|_| Changed::Unchanged);
        let mut has_change = false;
        for i in 0..N {
            match self[i].comparison(&other[i]) {
                Changed::Unchanged => (),
                Changed::Changed(v) => {
                    has_change = true;
                    result[i] = Changed::Changed(v)
                }
            }
        }
        if has_change {
            Changed::Changed(result)
        } else {
            Changed::Unchanged
        }
    }
}
