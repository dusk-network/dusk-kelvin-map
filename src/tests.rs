// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{KelvinMap, Map};

use core::borrow::Borrow;

use canonical::{Canon, Store};
use canonical_derive::Canon;
use canonical_host::MemStore;
use microkelvin::Cardinality;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

/// Simple key-value pair wrapper
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Canon)]
struct KeyValue {
    pub key: u64,
    pub value: u32,
}

impl KeyValue {
    fn random<R: RngCore>(rng: &mut R) -> Self {
        Self {
            key: rng.next_u64(),
            value: rng.next_u32(),
        }
    }

    fn generate_map<const L: usize>() -> (Vec<KeyValue>, Map<u64, u32, MemStore>)
    {
        // This seed will not generate duplicates
        let mut rng = StdRng::seed_from_u64(2321u64);
        let mut map = Map::default();

        // Create a set of dummy random KeyValue
        let mut data = vec![];

        for _ in 0..L {
            data.push(KeyValue::random(&mut rng));
        }

        data.iter().for_each(|d| {
            assert!(map.insert(d.key, d.value).unwrap().is_none());
            assert_eq!(d.value, *map.get(&d.key).unwrap().unwrap());
        });

        (data, map)
    }
}

fn assert_balanced<K, V, S>(map: &Map<K, V, S>)
where
    K: Canon<S> + Ord + Default,
    V: Canon<S>,
    S: Store,
{
    let (l, r) = match map {
        KelvinMap::Node(l, r) => (l, r),
        _ => panic!("Not possible to assert balance for a leaf or empty tree"),
    };

    let c_l: &Cardinality = l.annotation().borrow();
    let c_l: u64 = c_l.into();
    let c_l: i32 = c_l as i32;

    let c_r: &Cardinality = r.annotation().borrow();
    let c_r: u64 = c_r.into();
    let c_r: i32 = c_r as i32;

    // Assert they have equivalent cardinality for worst case scenario
    assert!((c_l - c_r).abs() <= 2);
}

#[test]
fn insert_get_mut() {
    let n = 16;

    let mut map: Map<u64, u64, MemStore> = Map::default();

    for i in 0..n {
        map.insert(i, i).unwrap();
    }

    for i in 0..n {
        *map.get_mut(&i).unwrap().unwrap() += 1;
    }

    assert_balanced(&map);

    for i in 0..n {
        assert_eq!(*map.get(&i).unwrap().unwrap(), i + 1)
    }
}

#[test]
fn remove_null() {
    // This seed will not generate duplicates
    let mut rng = StdRng::seed_from_u64(2321u64);
    let mut map: Map<u64, u32, MemStore> = Map::default();

    let kv = KeyValue::random(&mut rng);
    assert_eq!(Ok(None), map.remove(&kv.key));
}

#[test]
fn remove_single() {
    let (data, mut map) = KeyValue::generate_map::<1>();

    let v = map.remove(&data[0].key).unwrap().unwrap();
    assert_eq!(data[0].value, v);

    assert!(map.remove(&data[0].key).unwrap().is_none());
    assert!(map.is_empty());
}

#[test]
fn remove_multiple() {
    const L: usize = u8::MAX as usize;
    let (data, mut map) = KeyValue::generate_map::<L>();

    assert_balanced(&map);

    let mut k = (L - 2) as usize;
    while k > 0 {
        let v = map.remove(&data[k].key).unwrap().unwrap();

        assert_eq!(data[k].value, v);
        assert!(map.remove(&data[k].key).unwrap().is_none());

        k /= 2;
    }
}

#[test]
fn balance() {
    let mut map: Map<u8, u8, MemStore> = Map::default();

    // Ordered inserting is a worst case scenario for a BST
    for v in 0..130 {
        map.insert(v, v.wrapping_mul(3)).unwrap();
    }

    assert_balanced(&map);
}

#[test]
fn balance_rev() {
    let mut map: Map<u8, u8, MemStore> = Map::default();

    // Reverse order inserting is a worst case scenario for a BST
    for v in 0..130 {
        map.insert(130 - v, v.wrapping_mul(3)).unwrap();
    }

    assert_balanced(&map);
}
