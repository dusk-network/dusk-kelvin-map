// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Map;

use canonical::Canon;
use canonical_derive::Canon;
use canonical_host::MemStore;
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

    fn generate_map<const L: usize>() -> (Vec<KeyValue>, Map<u64, u32, MemStore>) {
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
            assert_eq!(d.value, map.get(&d.key).unwrap().unwrap());
        });

        (data, map)
    }
}

#[test]
fn insert_get_mutate() {
    // Generate a huge set
    const L: usize = i16::MAX as usize;
    let (data, mut map) = KeyValue::generate_map::<L>();

    data.iter().for_each(|d| {
        let k = d.key;
        let mut v = d.value;

        assert_eq!(v, map.get(&k).unwrap().unwrap());

        let old = v;
        v /= 2;
        assert_eq!(old, map.insert(k, v).unwrap().unwrap());
        assert_eq!(v, map.get(&k).unwrap().unwrap());

        v = v.saturating_add(1);
        let x = map
            .map_mut(&k, |x| {
                *x = x.saturating_add(1);
                *x
            })
            .unwrap()
            .unwrap();
        assert_eq!(v, x);
    });
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

    let mut k = (L - 2) as usize;
    while k > 0 {
        let v = map.remove(&data[k].key).unwrap().unwrap();

        assert_eq!(data[k].value, v);
        assert!(map.remove(&data[k].key).unwrap().is_none());

        k /= 2;
    }
}
