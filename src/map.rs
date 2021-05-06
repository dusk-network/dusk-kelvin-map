// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Leaf, MapAnnotation};

use core::ops::{Deref, DerefMut};
use core::{cmp, mem};

use canonical::{Canon, CanonError};
use canonical_derive::Canon;

use microkelvin::{
    Annotated, Branch, BranchMut, Cardinality, Child, ChildMut, Compound,
    MaxKey, Step, Walk, Walker,
};

#[derive(Debug, Clone, Canon)]
/// Binary tree map-like implementation with Microkelvin set as backend
///
/// The borrowed [`Max`] from the annotation will be used to traverse the tree and is expected to
/// be the maximum `K` contained in that sub-tree.
pub enum KelvinMap<K, V, A>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    /// Represents and empty endpoint
    Empty,
    /// Leaf of the tree containing a key -> value mapping
    Leaf(Leaf<K, V>),
    /// Annotated node that will contain, at least, the maximum key value that exists within this
    /// sub-tree
    Node(
        Annotated<KelvinMap<K, V, A>, A>,
        Annotated<KelvinMap<K, V, A>, A>,
    ),
}

impl<K, V, A> Default for KelvinMap<K, V, A>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    fn default() -> Self {
        KelvinMap::Empty
    }
}

impl<K, V, A> Compound<A> for KelvinMap<K, V, A>
where
    V: Canon,
    K: Canon + Ord,
    A: MapAnnotation<K, V>,
{
    type Leaf = Leaf<K, V>;

    fn child(&self, ofs: usize) -> Child<Self, A> {
        match (ofs, self) {
            (0, KelvinMap::Node(l, _)) => Child::Node(l),
            (1, KelvinMap::Node(_, r)) => Child::Node(r),
            (0, KelvinMap::Leaf(l)) => Child::Leaf(l),
            _ => Child::EndOfNode,
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, A> {
        match (ofs, self) {
            (0, KelvinMap::Node(l, _)) => ChildMut::Node(l),
            (1, KelvinMap::Node(_, r)) => ChildMut::Node(r),
            (0, KelvinMap::Leaf(l)) => ChildMut::Leaf(l),
            _ => ChildMut::EndOfNode,
        }
    }
}

// MaxKey doesn't implement PartialCmp<K>
fn cmp_max_key<K, V, A>(
    ann: &Annotated<KelvinMap<K, V, A>, A>,
    key: &K,
) -> cmp::Ordering
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    match ann.annotation().borrow() {
        MaxKey::Maximum(ann) => MaxKey::Maximum(ann).cmp(&MaxKey::Maximum(key)),
        MaxKey::NegativeInfinity => cmp::Ordering::Less,
    }
}

struct BinaryWalker<'a, K>(&'a K)
where
    K: Canon + Ord;

impl<'a, K, V, A> Walker<KelvinMap<K, V, A>, A> for BinaryWalker<'a, K>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    fn walk(&mut self, walk: Walk<KelvinMap<K, V, A>, A>) -> Step {
        match (walk.child(0), walk.child(1)) {
            // (0, 0) Empty tree
            (
                Child::EndOfNode | Child::Empty,
                Child::EndOfNode | Child::Empty,
            ) => Step::Abort,

            // (0, r) Invalid tree
            (
                Child::EndOfNode | Child::Empty,
                Child::Leaf(_) | Child::Node(_),
            ) => unreachable!(),

            // (_, r), r < k Key out of range
            (_, Child::Node(r)) if cmp_max_key(r, &self.0).is_lt() => {
                Step::Abort
            }

            // Key match
            (Child::Leaf(l), _) if l._key() == self.0 => Step::Found(0),
            (_, Child::Leaf(r)) if r._key() == self.0 => Step::Found(1),

            // End of path without match
            (
                Child::Leaf(_),
                Child::Leaf(_) | Child::EndOfNode | Child::Empty,
            ) => Step::Abort,

            // (l, _) l >= k Traverse left
            (Child::Node(l), _) if cmp_max_key(l, &self.0).is_ge() => {
                Step::Into(0)
            }

            // (_, r) Traverse right, k <= r is already tested
            (_, Child::Node(_)) => Step::Into(1),

            (
                Child::Node(_),
                Child::Empty | Child::EndOfNode | Child::Leaf(_),
            ) => Step::Abort,
        }
    }
}

/// Private struct used to hide the complex branch signature behind an
/// `impl Deref<Target = V>` for returning references to values in the map
struct ValRef<'a, K, V, A>(Branch<'a, KelvinMap<K, V, A>, A>)
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>;

impl<'a, K, V, A> Deref for ValRef<'a, K, V, A>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &**self.0
    }
}

/// Private struct used to hide the complex branch signature behind an
/// `impl DerefMut<Target = V>` for returning mutable references to values in the map
struct ValRefMut<'a, K, V, A>(BranchMut<'a, KelvinMap<K, V, A>, A>)
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>;

impl<'a, K, V, A> Deref for ValRefMut<'a, K, V, A>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &**self.0
    }
}

impl<'a, K, V, A> DerefMut for ValRefMut<'a, K, V, A>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut **self.0
    }
}

impl<K, V, A> KelvinMap<K, V, A>
where
    K: Canon + Ord,
    V: Canon,
    A: MapAnnotation<K, V>,
{
    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        match self {
            KelvinMap::Empty => 0,
            KelvinMap::Leaf(_) => 1,
            KelvinMap::Node(l, r) => {
                let c_l: &Cardinality = l.annotation().borrow();
                let c_l: u64 = c_l.into();
                let c_l = c_l as usize;

                let c_r: &Cardinality = r.annotation().borrow();
                let c_r: u64 = c_r.into();
                let c_r = c_r as usize;

                c_l + c_r
            }
        }
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        match self {
            KelvinMap::Empty => true,
            _ => false,
        }
    }

    /// Returns a reference to the value corresponding to the key
    ///
    /// Will return `Ok(None)` if no correspondent key was found.
    pub fn get<'a>(
        &'a self,
        k: &K,
    ) -> Result<Option<impl Deref<Target = V> + 'a>, CanonError> {
        Branch::walk(self, BinaryWalker(k))
            .map(|result| result.map(|branch| ValRef(branch)))
    }

    /// Returns a mutable reference to the value corresponding to the key
    ///
    /// Will return `Ok(None)` if no correspondent key was found.
    pub fn get_mut<'a>(
        &'a mut self,
        k: &K,
    ) -> Result<Option<impl DerefMut<Target = V> + 'a>, CanonError> {
        BranchMut::walk(self, BinaryWalker(k))
            .map(|result| result.map(|branch| ValRefMut(branch)))
    }

    /// Traverse the tree to find the minimum leaf-key
    fn min_key_leaf(&self) -> Result<Option<Leaf<K, V>>, CanonError> {
        match self {
            KelvinMap::Empty => Ok(None),
            KelvinMap::Leaf(l) => Ok(Some(l.clone())),
            KelvinMap::Node(l, _) => l.val()?.min_key_leaf(),
        }
    }

    /// Traverse the tree to find the maximum leaf-key
    fn max_key_leaf(&self) -> Result<Option<Leaf<K, V>>, CanonError> {
        match self {
            KelvinMap::Empty => Ok(None),
            KelvinMap::Leaf(l) => Ok(Some(l.clone())),
            KelvinMap::Node(_, r) => r.val()?.max_key_leaf(),
        }
    }

    /// Balance the map
    fn balance(&mut self) -> Result<(), CanonError> {
        let (l, r) = match self {
            KelvinMap::Node(l, r) => (l, r),
            _ => return Ok(()),
        };

        let c_l: &Cardinality = l.annotation().borrow();
        let c_l: u64 = c_l.into();

        let c_r: &Cardinality = r.annotation().borrow();
        let c_r: u64 = c_r.into();

        // TODO - Improve the performance with a tree rotation
        let left_leaf = l.val_mut()?.max_key_leaf()?;
        let right_leaf = r.val()?.min_key_leaf()?;
        match (left_leaf, right_leaf) {
            (_, Some(leaf)) if c_r > c_l.saturating_add(1) => {
                r.val_mut()?._remove(leaf._key())?;
                l.val_mut()?._insert(leaf)?;
            }

            (Some(leaf), _) if c_l > c_r.saturating_add(1) => {
                l.val_mut()?._remove(leaf._key())?;
                r.val_mut()?._insert(leaf)?;
            }

            _ => (),
        }

        Ok(())
    }

    /// Remove a key -> value mapping from the set.
    ///
    /// If the key was previously mapped, it will return the old value in the form `Ok(Some(V))`.
    ///
    /// If the key was not previously mapped, the return will be `Ok(None)`. This operation is
    /// idempotent.
    ///
    /// Internally, a naive balancing will be performed. If the tree contains more elements on the
    /// left, it will move the maximum key of the left to the right - and vice-versa.
    pub fn remove(&mut self, k: &K) -> Result<Option<V>, CanonError> {
        self.balance()?;

        self._remove(k)
    }

    fn _remove(&mut self, k: &K) -> Result<Option<V>, CanonError> {
        match self {
            KelvinMap::Empty => Ok(None),

            KelvinMap::Leaf(leaf) if leaf._key() == k => {
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
                    if leaf._key() == k {
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
                    if leaf._key() == k {
                        old.replace(leaf.value().clone());
                    }
                }

                if old.is_some() {
                    let new = mem::take(&mut *l.val_mut()?);
                    *self = new;
                    return Ok(old);
                }

                if cmp_max_key(l, k).is_ge() {
                    l.val_mut()?.remove(k)
                } else if cmp_max_key(r, k).is_ge() {
                    r.val_mut()?.remove(k)
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Include a key -> value mapping to the set.
    ///
    /// If the key was previously mapped, it will return the old value in the form `Ok(Some(V))`.
    ///
    /// If the key was not previously mapped, the return will be `Ok(None)`
    ///
    /// Internally, a naive balancing will be performed. If the tree contains more elements on the
    /// left, it will move the maximum key of the left to the right - and vice-versa.
    pub fn insert(&mut self, k: K, v: V) -> Result<Option<V>, CanonError> {
        let leaf = Leaf::new(k, v);

        self.balance()?;

        self._insert(leaf)
    }

    fn _insert(&mut self, leaf: Leaf<K, V>) -> Result<Option<V>, CanonError> {
        let mut old = None;

        match self {
            KelvinMap::Empty => *self = KelvinMap::Leaf(leaf),

            KelvinMap::Leaf(l) if l._key() == leaf._key() => {
                old.replace(l.value().clone());
                *self = KelvinMap::Leaf(leaf);
            }

            KelvinMap::Leaf(l) if l._key() < leaf._key() => {
                let left = Annotated::new(mem::take(self));
                let right = Annotated::new(KelvinMap::Leaf(leaf));

                *self = KelvinMap::Node(left, right);
            }

            KelvinMap::Leaf(l) if leaf._key() < l._key() => {
                let left = Annotated::new(KelvinMap::Leaf(leaf));
                let right = Annotated::new(mem::take(self));

                *self = KelvinMap::Node(left, right);
            }

            KelvinMap::Node(l, _) if cmp_max_key(l, leaf._key()).is_ge() => {
                old = l.val_mut()?._insert(leaf)?;
            }

            KelvinMap::Node(l, r) if cmp_max_key(l, leaf._key()).is_lt() => {
                old = r.val_mut()?._insert(leaf)?;
            }

            _ => return Err(CanonError::InvalidEncoding),
        }

        Ok(old)
    }
}
