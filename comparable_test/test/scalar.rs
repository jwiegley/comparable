#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_cast)]

use comparable::{Changed::*, *};

#[test]
fn test_unit() {
	assert_changes!(&(), &(), Unchanged);
}

#[test]
fn test_bool() {
	assert_changes!(&(), &(), Unchanged);
}

#[test]
fn test_u8() {
	assert_changes!(&(100 as u8), &(100 as u8), Unchanged);
	assert_changes!(&(100 as u8), &(200 as u8), Changed(U8Change(100, 200)));
}

#[test]
fn test_i8() {
	assert_changes!(&(100 as i8), &(100 as i8), Unchanged);
	assert_changes!(&(100 as i8), &(120 as i8), Changed(I8Change(100, 120)));
}

#[test]
fn test_u16() {
	assert_changes!(&(100 as u16), &(100 as u16), Unchanged);
	assert_changes!(&(100 as u16), &(200 as u16), Changed(U16Change(100, 200)));
}

#[test]
fn test_i16() {
	assert_changes!(&(100 as i16), &(100 as i16), Unchanged);
	assert_changes!(&(100 as i16), &(200 as i16), Changed(I16Change(100, 200)));
}

#[test]
fn test_u32() {
	assert_changes!(&(100 as u32), &(100 as u32), Unchanged);
	assert_changes!(&(100 as u32), &(200 as u32), Changed(U32Change(100, 200)));
}

#[test]
fn test_i32() {
	assert_changes!(&(100 as i32), &(100 as i32), Unchanged);
	assert_changes!(&(100 as i32), &(200 as i32), Changed(I32Change(100, 200)));
}

#[test]
fn test_u64() {
	assert_changes!(&(100 as u64), &(100 as u64), Unchanged);
	assert_changes!(&(100 as u64), &(200 as u64), Changed(U64Change(100, 200)));
}

#[test]
fn test_i64() {
	assert_changes!(&(100 as i64), &(100 as i64), Unchanged);
	assert_changes!(&(100 as i64), &(200 as i64), Changed(I64Change(100, 200)));
}

#[test]
fn test_usize() {
	assert_changes!(&(100 as usize), &(100 as usize), Unchanged);
	assert_changes!(&(100 as usize), &(200 as usize), Changed(UsizeChange(100, 200)),);
}

#[test]
fn test_isize() {
	assert_changes!(&(100 as isize), &(100 as isize), Unchanged);
	assert_changes!(&(100 as isize), &(200 as isize), Changed(IsizeChange(100, 200)),);
}

#[test]
fn test_f32() {
	assert_changes!(&(100.0 as f32), &(100.0 as f32), Unchanged);
	assert_changes!(&(100.0 as f32), &(200.0 as f32), Changed(F32Change(100.0, 200.0)),);
}

#[test]
fn test_f64() {
	assert_changes!(&100.0 as &f64, &100.0 as &f64, Unchanged);
	assert_changes!(&100.0 as &f64, &200.0 as &f64, Changed(F64Change(100.0, 200.0)),);
}

#[test]
fn test_char() {
	assert_changes!(&'a', &'a', Unchanged);
	assert_changes!(&'a', &'b', Changed(CharChange('a', 'b')));
}
