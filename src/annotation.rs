// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{KelvinMap, Leaf};

use canonical::Canon;
use canonical_derive::Canon;
use microkelvin::{Annotation, Cardinality, Combine, MaxKey};

use core::borrow::Borrow;

/// Trait requirement to be an annotation of `KelvinMap`.
///
/// The borrowed `Max<K>` will be used to define the traversal path over the tree.
pub trait MapAnnotation<K, V>
where
    K: Canon + Ord,
    V: Canon,
    Self: Canon + Annotation<Leaf<K, V>> + Combine<KelvinMap<K, V, Self>, Self>,
    Self: Borrow<MaxKey<K>> + Borrow<Cardinality>,
{
}

#[derive(Debug, Clone, Default, Canon)]
/// Minimum working annotation for the KelvinMap.
///
/// Internally contains a [`Max`] implementation for `K`.
///
/// The `Default` implementation of `K` will be considered as the negative infinity for the `Max`
/// annotation.
pub struct MapAnnotationDefault<K>
where
    K: Canon + Ord + Default,
{
    cardinality: Cardinality,
    max: MaxKey<K>,
}

impl<K> Borrow<MaxKey<K>> for MapAnnotationDefault<K>
where
    K: Canon + Ord + Default,
{
    fn borrow(&self) -> &MaxKey<K> {
        &self.max
    }
}

impl<K> Borrow<Cardinality> for MapAnnotationDefault<K>
where
    K: Canon + Ord + Default,
{
    fn borrow(&self) -> &Cardinality {
        &self.cardinality
    }
}

impl<K, V> Annotation<Leaf<K, V>> for MapAnnotationDefault<K>
where
    K: Canon + Ord + Default,
{
    fn from_leaf(leaf: &Leaf<K, V>) -> Self {
        let cardinality = Cardinality::from_leaf(leaf);
        let max = MaxKey::from_leaf(leaf);

        Self { cardinality, max }
    }
}

impl<K, V>
    Combine<KelvinMap<K, V, MapAnnotationDefault<K>>, MapAnnotationDefault<K>>
    for MapAnnotationDefault<K>
where
    K: Canon + Ord + Default,
    V: Canon,
{
    fn combine(node: &KelvinMap<K, V, MapAnnotationDefault<K>>) -> Self {
        let cardinality = Cardinality::combine(node);
        let max = MaxKey::combine(node);

        Self { cardinality, max }
    }
}

impl<K, V> MapAnnotation<K, V> for MapAnnotationDefault<K>
where
    K: Canon + Ord + Default,
    V: Canon,
{
}
