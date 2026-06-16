//! Criterion benchmarks for the core `describe` and `comparison` operations.
//!
//! These exercise the paths the library actually cares about: describing a
//! value, comparing two equal values (the common "nothing changed" case in a
//! passing test), comparing values that differ in a field, and diffing large
//! collections.

#![allow(clippy::derive_partial_eq_without_eq)]

use std::collections::BTreeMap;

use comparable::Comparable;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Clone, comparable_derive::Comparable)]
struct Inner {
	a: u64,
	b: String,
	flags: Vec<bool>,
}

#[derive(Clone, comparable_derive::Comparable)]
struct Record {
	id: u64,
	name: String,
	values: Vec<i64>,
	inner: Inner,
	attributes: BTreeMap<u32, String>,
}

fn sample(n: usize) -> Record {
	Record {
		id: n as u64,
		name: format!("record-{n}"),
		values: (0..n as i64).collect(),
		inner: Inner { a: n as u64, b: "inner".to_string(), flags: (0..n).map(|i| i % 2 == 0).collect() },
		attributes: (0..n as u32).map(|i| (i, format!("v{i}"))).collect(),
	}
}

fn bench_describe(c: &mut Criterion) {
	let record = sample(256);
	c.bench_function("describe/record_256", |b| b.iter(|| black_box(&record).describe()));
}

fn bench_comparison_unchanged(c: &mut Criterion) {
	let record = sample(256);
	c.bench_function("comparison/record_256_unchanged", |b| {
		b.iter(|| black_box(&record).comparison(black_box(&record)))
	});
}

fn bench_comparison_changed(c: &mut Criterion) {
	let before = sample(256);
	let mut after = before.clone();
	after.inner.a += 1;
	*after.values.last_mut().unwrap() += 1;
	c.bench_function("comparison/record_256_changed", |b| b.iter(|| black_box(&before).comparison(black_box(&after))));
}

fn bench_comparison_vec(c: &mut Criterion) {
	let before: Vec<i64> = (0..1024).collect();
	let mut after = before.clone();
	after[512] = -1;
	c.bench_function("comparison/vec_1024", |b| b.iter(|| black_box(&before).comparison(black_box(&after))));
}

fn bench_comparison_map(c: &mut Criterion) {
	let before: BTreeMap<u32, String> = (0..512).map(|i| (i, format!("v{i}"))).collect();
	let mut after = before.clone();
	after.insert(256, "changed".to_string());
	after.remove(&100);
	c.bench_function("comparison/map_512", |b| b.iter(|| black_box(&before).comparison(black_box(&after))));
}

criterion_group!(
	benches,
	bench_describe,
	bench_comparison_unchanged,
	bench_comparison_changed,
	bench_comparison_vec,
	bench_comparison_map
);
criterion_main!(benches);
