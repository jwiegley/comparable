use comparable::{Changed::*, *};
use std::path::{Path, PathBuf};

// Regression test for https://github.com/jwiegley/comparable/issues/13:
// `Path`/`PathBuf` lost their `Comparable` impl in 0.5.6 because `pub mod
// path;` (and the `PathBufChange` re-export) was dropped from the generated
// `lib.rs`.

#[test]
fn test_pathbuf_unchanged() {
	let p1 = PathBuf::from("/foo/bar");
	let p2 = PathBuf::from("/foo/bar");
	assert_changes!(&p1, &p2, Unchanged);
}

#[test]
fn test_pathbuf_changed() {
	let p1 = PathBuf::from("/foo/bar");
	let p2 = PathBuf::from("/foo/baz");
	assert_changes!(&p1, &p2, Changed(PathBufChange(PathBuf::from("/foo/bar"), PathBuf::from("/foo/baz"))));
}

#[test]
fn test_pathbuf_describe() {
	let p = PathBuf::from("/foo/bar");
	assert_eq!(p.describe(), PathBuf::from("/foo/bar"));
}

#[test]
fn test_path_unchanged() {
	let p1 = PathBuf::from("/foo/bar");
	let p2 = PathBuf::from("/foo/bar");
	assert_changes!(p1.as_path(), p2.as_path(), Unchanged);
}

#[test]
fn test_path_changed() {
	let p1 = PathBuf::from("/foo/bar");
	let p2 = PathBuf::from("/foo/baz");
	assert_changes!(
		p1.as_path(),
		p2.as_path(),
		Changed(PathBufChange(PathBuf::from("/foo/bar"), PathBuf::from("/foo/baz")))
	);
}

#[test]
fn test_path_describe() {
	let p: &Path = Path::new("/foo/bar");
	assert_eq!(p.describe(), PathBuf::from("/foo/bar"));
}
