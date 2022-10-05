#![allow(dead_code)]
#![deny(warnings)]

use imhamt::*;
use std::collections::hash_map::DefaultHasher;

fn main() {}

fn check_insert(map: Hamt<DefaultHasher, i32, i32>) {
    map.insert(1, 2).unwrap();
}
fn check_remove(map: Hamt<DefaultHasher, i32, i32>) {
    map.remove(&1).unwrap();
}
fn check_remove_match(map: Hamt<DefaultHasher, i32, i32>) {
    map.remove_match(&1, &2).unwrap();
}
fn check_replace(map: Hamt<DefaultHasher, i32, i32>) {
    map.replace(&1, 2).unwrap();
}
fn check_update(map: Hamt<DefaultHasher, i32, i32>) {
    map.update(&1, update).unwrap();
}
fn check_insert_or_update(map: Hamt<DefaultHasher, i32, i32>) {
    map.insert_or_update(1, 2, update).unwrap();
}
// fn check_insert_or_update_simple(map: Hamt<DefaultHasher, i32, i32>) {
//     map.insert_or_update_simple(1, 2, update_simple);
// }

fn update(x: &i32) -> Result<Option<i32>, std::io::Error> {
    Ok(Some(*x))
}

fn update_simple(x: &i32) -> Option<i32> {
    Some(*x)
}
