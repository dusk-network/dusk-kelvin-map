// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{KelvinMap, Leaf};

use canonical::{Canon, Store};
use canonical_derive::Canon;
use microkelvin::{Annotation, Cardinality, Max};

use core::borrow::Borrow;
use core::marker::PhantomData;

/// Trait requirement to be an annotation of `KelvinMap`.
///
/// The borrowed `Max<K>` will be used to define the traversal path over the tree.
pub trait MapAnnotation<K, V, S, const N: usize>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    S: Store,
    Self: Canon<S> + Annotation<KelvinMap<K, V, Self, S, N>, S>,
    Self: Borrow<Max<K>> + Borrow<Cardinality>,
{
}

#[derive(Debug, Clone, Canon)]
/// Minimum working annotation for the KelvinMap.
///
/// Internally contains a [`Max`] implementation for `K`.
///
/// The `Default` implementation of `K` will be considered as the negative infinity for the `Max`
/// annotation.
pub struct MapAnnotationDefault<K, S>
where
    K: Canon<S> + Ord + Default,
    S: Store,
{
    cardinality: Cardinality,
    max: Max<K>,
    store: PhantomData<S>,
}

impl<K, V, S, const N: usize> MapAnnotation<K, V, S, N>
    for MapAnnotationDefault<K, S>
where
    K: Canon<S> + Ord + Default,
    V: Canon<S>,
    S: Store,
{
}

impl<K, S> Borrow<Max<K>> for MapAnnotationDefault<K, S>
where
    K: Canon<S> + Ord + Default,
    S: Store,
{
    fn borrow(&self) -> &Max<K> {
        &self.max
    }
}

impl<K, S> Borrow<Cardinality> for MapAnnotationDefault<K, S>
where
    K: Canon<S> + Ord + Default,
    S: Store,
{
    fn borrow(&self) -> &Cardinality {
        &self.cardinality
    }
}

impl<K, V, S, const N: usize>
    Annotation<KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>, S>
    for MapAnnotationDefault<K, S>
where
    K: Canon<S> + Ord + Default,
    V: Canon<S>,
    S: Store,
{
    fn identity() -> Self {
        let cardinality = <Cardinality as Annotation<
            KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>,
            S,
        >>::identity();
        let max = Max::Maximum(K::default());

        Self {
            cardinality,
            max,
            store: PhantomData,
        }
    }

    fn from_leaf(leaf: &Leaf<K, V>) -> Self {
        let cardinality = <Cardinality as Annotation<
            KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>,
            S,
        >>::from_leaf(leaf);
        let max = <Max<K> as Annotation<
            KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>,
            S,
        >>::from_leaf(leaf);

        Self {
            cardinality,
            max,
            store: PhantomData,
        }
    }

    fn from_node(
        node: &KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>,
    ) -> Self {
        let cardinality = <Cardinality as Annotation<
            KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>,
            S,
        >>::from_node(node);
        let max = <Max<K> as Annotation<
            KelvinMap<K, V, MapAnnotationDefault<K, S>, S, N>,
            S,
        >>::from_node(node);

        Self {
            cardinality,
            max,
            store: PhantomData,
        }
    }
}
