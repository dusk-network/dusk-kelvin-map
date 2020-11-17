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
}

#[test]
fn insert_get_mutate() {
    // This seed will not generate duplicates
    let mut rng = StdRng::seed_from_u64(2321u64);
    let mut map: Map<u64, u32, MemStore> = Map::default();

    // Create a huge set of dummy random KeyValue
    const M: usize = u16::max_value() as usize;
    let mut data = [KeyValue::default(); M];
    data.iter_mut()
        .for_each(|l| *l = KeyValue::random(&mut rng));

    data.iter().for_each(|d| {
        assert!(map.insert(d.key, d.value).unwrap().is_none());
        assert_eq!(d.value, map.get(&d.key).unwrap().unwrap());
    });

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
