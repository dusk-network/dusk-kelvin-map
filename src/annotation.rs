// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{KelvinMap, Leaf};

use canonical::{Canon, Store};
use canonical_derive::Canon;
use microkelvin::{Annotation, Max};

use core::borrow::Borrow;
use core::marker::PhantomData;

#[derive(Debug, Clone, Canon)]
/// Minimum working annotation for the KelvinMap.
///
/// Internally contains a [`Max`] implementation for `K`.
pub struct MapAnnotation<K, S>
where
    K: Canon<S> + PartialOrd + Default,
    S: Store,
{
    max: Max<K>,
    store: PhantomData<S>,
}

impl<K, S> Borrow<Max<K>> for MapAnnotation<K, S>
where
    K: Canon<S> + PartialOrd + Default,
    S: Store,
{
    fn borrow(&self) -> &Max<K> {
        &self.max
    }
}

impl<K, S> Borrow<K> for MapAnnotation<K, S>
where
    K: Canon<S> + PartialOrd + Default,
    S: Store,
{
    fn borrow(&self) -> &K {
        match &self.max {
            // The identity is defined as the default value of K
            Max::NegativeInfinity => unreachable!(),
            Max::Maximum(m) => m,
        }
    }
}

impl<K, V, S> Annotation<KelvinMap<K, V, MapAnnotation<K, S>, S>, S> for MapAnnotation<K, S>
where
    K: Canon<S> + PartialOrd + Default,
    V: Canon<S>,
    S: Store,
{
    fn identity() -> Self {
        let max = Max::Maximum(K::default());

        Self {
            max,
            store: PhantomData,
        }
    }

    fn from_leaf(leaf: &Leaf<K, V>) -> Self {
        let max =
            <Max<K> as Annotation<KelvinMap<K, V, MapAnnotation<K, S>, S>, S>>::from_leaf(leaf);

        Self {
            max,
            store: PhantomData,
        }
    }

    fn from_node(node: &KelvinMap<K, V, MapAnnotation<K, S>, S>) -> Self {
        let max =
            <Max<K> as Annotation<KelvinMap<K, V, MapAnnotation<K, S>, S>, S>>::from_node(node);

        Self {
            max,
            store: PhantomData,
        }
    }
}
