// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::Canon;
use canonical_derive::Canon;

use core::borrow::Borrow;
use core::cmp::Ordering;

#[derive(Debug, Clone, Canon)]
/// Wrapper for the key -> value mapping the will act as leaf of the tree
pub struct Leaf<K, V>
where
    K: Ord,
{
    key: K,
    value: V,
}

impl<K, V> Leaf<K, V>
where
    K: Ord,
{
    pub(crate) fn new(key: K, value: V) -> Self {
        Self { key, value }
    }

    /// Stored key of the key -> value mapping
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Stored value of the key -> value mapping
    pub fn value(&self) -> &V {
        &self.value
    }

    /// Mutable reference to the stored value of the key -> value mapping
    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

impl<K, V> PartialEq<K> for Leaf<K, V>
where
    K: Ord,
{
    fn eq(&self, rhs: &K) -> bool {
        self.key.eq(rhs)
    }
}

impl<K, V> PartialOrd<K> for Leaf<K, V>
where
    K: Ord,
{
    fn partial_cmp(&self, rhs: &K) -> Option<Ordering> {
        self.key.partial_cmp(rhs)
    }
}

impl<K, V> PartialEq for Leaf<K, V>
where
    K: Ord,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.key.eq(&rhs.key)
    }
}

impl<K, V> PartialOrd for Leaf<K, V>
where
    K: Ord,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&rhs.key)
    }
}

impl<K, V> Borrow<K> for Leaf<K, V>
where
    K: Ord,
{
    fn borrow(&self) -> &K {
        &self.key
    }
}
