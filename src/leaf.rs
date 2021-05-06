// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::borrow::Borrow;
use core::ops::{Deref, DerefMut};

use canonical_derive::Canon;
use microkelvin::Keyed;

#[derive(Debug, Clone, Canon)]
/// Wrapper for the key -> value mapping the will act as leaf of the tree
pub struct Leaf<K, V> {
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

    /// Stored key of the key -> value mapping as concrete representation
    pub(crate) fn _key(&self) -> &K {
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

impl<K, V> Keyed<K> for Leaf<K, V>
where
    K: Ord,
{
    fn key(&self) -> &K {
        &self.key
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

impl<K, V> Deref for Leaf<K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K, V> DerefMut for Leaf<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
