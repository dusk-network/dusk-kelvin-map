// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Leaf, MapAnnotation};

use canonical::{Canon, InvalidEncoding, Store};
use canonical_derive::Canon;
use microkelvin::{Annotated, Child, ChildMut, Compound, Max};

use core::borrow::Borrow;
use core::mem;

#[derive(Debug, Clone, Canon)]
/// Binary tree map-like implementation with Microkelvin set as backend
///
/// The borrowed [`Max`] from the annotation will be used to traverse the tree and is expected to
/// be the maximum `K` contained in that sub-tree.
pub enum KelvinMap<K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: MapAnnotation<K, V, S>,
    S: Store,
{
    /// Represents and empty endpoint
    Empty,
    /// Leaf of the tree containing a key -> value mapping
    Leaf(Leaf<K, V>),
    /// Annotated node that will contain, at least, the maximum key value that exists within this
    /// sub-tree
    Node(
        Annotated<KelvinMap<K, V, A, S>, S>,
        Annotated<KelvinMap<K, V, A, S>, S>,
    ),
}

impl<K, V, A, S> Default for KelvinMap<K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: MapAnnotation<K, V, S>,
    S: Store,
{
    fn default() -> Self {
        KelvinMap::Empty
    }
}

impl<K, V, A, S> Compound<S> for KelvinMap<K, V, A, S>
where
    V: Canon<S>,
    K: Canon<S> + Ord,
    A: MapAnnotation<K, V, S>,
    S: Store,
{
    type Leaf = Leaf<K, V>;
    type Annotation = A;

    fn child(&self, ofs: usize) -> Child<Self, S> {
        match (ofs, self) {
            (0, KelvinMap::Node(l, _)) => Child::Node(l),
            (1, KelvinMap::Node(_, r)) => Child::Node(r),
            (0, KelvinMap::Leaf(l)) => Child::Leaf(l),
            _ => Child::EndOfNode,
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, S> {
        match (ofs, self) {
            (0, KelvinMap::Node(l, _)) => ChildMut::Node(l),
            (1, KelvinMap::Node(_, r)) => ChildMut::Node(r),
            (0, KelvinMap::Leaf(l)) => ChildMut::Leaf(l),
            _ => ChildMut::EndOfNode,
        }
    }
}

#[inline]
fn borrow_max<K, A: Borrow<Max<K>>>(ann: &A) -> &Max<K> {
    // Borrow does not accept generic parameters; this is a helper to relax the type resolution of
    // the compiler
    ann.borrow()
}

impl<K, V, A, S> KelvinMap<K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: MapAnnotation<K, V, S>,
    S: Store,
{
    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        match self {
            KelvinMap::Empty => true,
            _ => false,
        }
    }

    /// Fetch a previously inserted key -> value mapping, provided the key.
    ///
    /// Will return `Ok(None)` if no correspondent key was found.
    pub fn get(&self, k: &K) -> Result<Option<V>, S::Error> {
        match self {
            KelvinMap::Empty => Ok(None),
            KelvinMap::Leaf(l) if l == k => Ok(Some(l.value().clone())),
            KelvinMap::Leaf(l) if l != k => Ok(None),
            KelvinMap::Node(l, _) if borrow_max(l.annotation()) >= k => l.val()?.get(k),
            KelvinMap::Node(l, r) if borrow_max(l.annotation()) < k => r.val()?.get(k),
            _ => Err(InvalidEncoding.into()),
        }
    }

    /// Mutably map the value of a previously inserted key -> value mapping.
    ///
    /// The provided `FnMut` will receive a mutable reference to the already found value and will
    /// expect `R` as return.
    ///
    /// Any changes performed to this mutable reference will be stored on the map.
    ///
    /// If the key was not previously found and no valid value can be sent to `f`, then `Ok(None)`
    /// will be returned.
    pub fn map_mut<F, R>(&mut self, k: &K, mut f: F) -> Result<Option<R>, S::Error>
    where
        F: FnMut(&mut V) -> R,
    {
        match self {
            KelvinMap::Empty => Ok(None),
            KelvinMap::Leaf(l) if l == k => Ok(Some(f(l.value_mut()))),
            KelvinMap::Leaf(l) if l != k => Ok(None),
            KelvinMap::Node(l, _) if borrow_max(l.annotation()) >= k => l.val_mut()?.map_mut(k, f),
            KelvinMap::Node(l, r) if borrow_max(l.annotation()) < k => r.val_mut()?.map_mut(k, f),
            _ => Err(InvalidEncoding.into()),
        }
    }

    /// Include a key -> value mapping to the set.
    ///
    /// If the key was previously mapped, it will return the old value in the form `Ok(Some(V))`.
    ///
    /// If the key was not previously mapped, the return will be `Ok(None)`
    pub fn insert(&mut self, k: K, v: V) -> Result<Option<V>, S::Error> {
        self._insert(Leaf::new(k, v))
    }

    fn _insert(&mut self, leaf: Leaf<K, V>) -> Result<Option<V>, S::Error> {
        let mut old = None;

        match self {
            KelvinMap::Empty => *self = KelvinMap::Leaf(leaf),

            KelvinMap::Leaf(l) if *l == leaf => {
                old.replace(l.value().clone());
                *self = KelvinMap::Leaf(leaf);
            }

            KelvinMap::Leaf(l) if *l < leaf => {
                let left = Annotated::new(mem::take(self));
                let right = Annotated::new(KelvinMap::Leaf(leaf));

                *self = KelvinMap::Node(left, right);
            }

            KelvinMap::Leaf(l) if leaf < *l => {
                let left = Annotated::new(KelvinMap::Leaf(leaf));
                let right = Annotated::new(mem::take(self));

                *self = KelvinMap::Node(left, right);
            }

            KelvinMap::Node(l, _) if borrow_max(l.annotation()) >= leaf.key() => {
                old = l.val_mut()?._insert(leaf)?;
            }

            KelvinMap::Node(l, r) if borrow_max(l.annotation()) < leaf.key() => {
                old = r.val_mut()?._insert(leaf)?;
            }

            _ => return Err(InvalidEncoding.into()),
        }

        Ok(old)
    }

    /// Remove a key -> value mapping from the set.
    ///
    /// If the key was previously mapped, it will return the old value in the form `Ok(Some(V))`.
    ///
    /// If the key was not previously mapped, the return will be `Ok(None)`. This operation is
    /// idempotent.
    pub fn remove(&mut self, k: &K) -> Result<Option<V>, S::Error> {
        match self {
            KelvinMap::Empty => Ok(None),

            KelvinMap::Leaf(leaf) if leaf.key() == k => {
                let old = Some(leaf.value().clone());

                *self = KelvinMap::Empty;

                Ok(old)
            }
            KelvinMap::Leaf(_) => Ok(None),

            KelvinMap::Node(l, r) => {
                let mut old = None;

                // If the key is the left child, take its value and move the right child to current
                // node
                if let KelvinMap::Leaf(leaf) = &mut *l.val_mut()? {
                    if leaf.key() == k {
                        old.replace(leaf.value().clone());
                    }
                }

                if old.is_some() {
                    let new = mem::take(&mut *r.val_mut()?);
                    *self = new;
                    return Ok(old);
                }

                // If the key is the right child, take its value and move the left child to current
                // node
                if let KelvinMap::Leaf(leaf) = &mut *r.val_mut()? {
                    if leaf.key() == k {
                        old.replace(leaf.value().clone());
                    }
                }

                if old.is_some() {
                    let new = mem::take(&mut *l.val_mut()?);
                    *self = new;
                    return Ok(old);
                }

                // Traverse the tree
                if borrow_max(l.annotation()) >= k {
                    l.val_mut()?.remove(k)
                } else if borrow_max(r.annotation()) >= k {
                    r.val_mut()?.remove(k)
                } else {
                    Ok(None)
                }
            }
        }
    }
}
