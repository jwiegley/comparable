use crate::types::{Changed, Comparable};
use std::path::{Path, PathBuf};

#[derive(Eq, PartialEq, Debug)]
pub struct PathBufChange(pub PathBuf, pub PathBuf);

impl Comparable for PathBuf {
    type Desc = PathBuf;

    fn describe(&self) -> Self::Desc {
        self.clone()
    }

    type Change = PathBufChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(PathBufChange(self.clone(), other.clone()))
        } else {
            Changed::Unchanged
        }
    }
}

impl Comparable for Path {
    type Desc = PathBuf;

    fn describe(&self) -> Self::Desc {
        self.to_path_buf()
    }

    type Change = PathBufChange;

    fn comparison(&self, other: &Self) -> Changed<Self::Change> {
        if self != other {
            Changed::Changed(PathBufChange(self.to_path_buf(), other.to_path_buf()))
        } else {
            Changed::Unchanged
        }
    }
}
