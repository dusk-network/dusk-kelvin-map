// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Leaf;

use core::borrow::Borrow;
use core::mem;
use core::ops::{Deref, DerefMut};

use canonical::{Canon, InvalidEncoding, Store};
use canonical_derive::Canon;
use microkelvin::{
    Annotated, Annotation, Branch, BranchMut, Cardinality, Child, ChildMut,
    Compound, Max, Step, StepMut, Walk, WalkMut,
};

#[derive(Debug, Clone, Canon)]
/// Binary tree map-like implementation with Microkelvin set as backend
///
/// The borrowed [`Max`] from the annotation will be used to traverse the tree and is expected to
/// be the maximum `K` contained in that sub-tree.
pub enum KelvinMap<K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S>,
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
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S>,
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
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S>,
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
    A: Canon<S>
        + Annotation<KelvinMap<K, V, A, S>, S>
        + Borrow<Cardinality>
        + Borrow<Max<K>>,
    S: Store,
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
    ) -> Result<Option<impl Deref<Target = V> + 'a>, S::Error> {
        Branch::<'a, _, _>::walk(self, |f| match f {
            Walk::Leaf(l) => {
                if l.key() == k {
                    Step::Found(l)
                } else {
                    Step::Next
                }
            }
            Walk::Node(n) => {
                if borrow_max(n.annotation()) >= k {
                    Step::Into(n)
                } else {
                    Step::Next
                }
            }
        })
        .map(|result| result.map(|branch| ValRef(branch)))
    }

    /// Returns a mutable reference to the value corresponding to the key
    ///
    /// Will return `Ok(None)` if no correspondent key was found.
    pub fn get_mut<'a>(
        &'a mut self,
        k: &K,
    ) -> Result<Option<impl DerefMut<Target = V> + 'a>, S::Error> {
        BranchMut::<'a, _, _>::walk(self, |f| match f {
            WalkMut::Leaf(l) => {
                if l.key() == k {
                    StepMut::Found(l)
                } else {
                    StepMut::Next
                }
            }
            WalkMut::Node(n) => {
                if borrow_max(n.annotation()) >= k {
                    StepMut::Into(n)
                } else {
                    StepMut::Next
                }
            }
        })
        .map(|result| result.map(|branch| ValRefMut(branch)))
    }

    /// Traverse the tree to find the minimum leaf-key
    fn min_key_leaf(&self) -> Result<Option<Leaf<K, V>>, S::Error> {
        match self {
            KelvinMap::Empty => Ok(None),
            KelvinMap::Leaf(l) => Ok(Some(l.clone())),

            KelvinMap::Node(l, _) => l.val()?.min_key_leaf(),
        }
    }

    /// Traverse the tree to find the maximum leaf-key
    fn max_key_leaf(&self) -> Result<Option<Leaf<K, V>>, S::Error> {
        match self {
            KelvinMap::Empty => Ok(None),
            KelvinMap::Leaf(l) => Ok(Some(l.clone())),
            KelvinMap::Node(_, r) => r.val()?.max_key_leaf(),
        }
    }

    /// Balance the map
    fn balance(&mut self) -> Result<(), S::Error> {
        let (l, r) = match self {
            KelvinMap::Node(l, r) => (l, r),
            _ => return Ok(()),
        };

        let c_l: &Cardinality = l.annotation().borrow();
        let c_l: u64 = c_l.into();

        let c_r: &Cardinality = r.annotation().borrow();
        let c_r: u64 = c_r.into();

        // TODO - Improve the performance with a tree rotation
        if c_r > c_l.saturating_add(1) {
            // Find the smallest element in `r`, remove it and append to `l`
            if let Some(leaf) = r.val()?.min_key_leaf()? {
                r.val_mut()?._remove(leaf.key())?;
                l.val_mut()?._insert(leaf)?;
            }
        } else if c_l > c_r.saturating_add(1) {
            // Find the biggest element in `l`, remove it and append to `r`
            if let Some(leaf) = l.val()?.max_key_leaf()? {
                l.val_mut()?._remove(leaf.key())?;
                r.val_mut()?._insert(leaf)?;
            }
        }

        Ok(())
    }

    /// Include a key -> value mapping to the set.
    ///
    /// If the key was previously mapped, it will return the old value in the form `Ok(Some(V))`.
    ///
    /// If the key was not previously mapped, the return will be `Ok(None)`
    ///
    /// Internally, a naive balancing will be performed. If the tree contains more elements on the
    /// left, it will move the maximum key of the left to the right - and vice-versa.
    pub fn insert(&mut self, k: K, v: V) -> Result<Option<V>, S::Error> {
        let leaf = Leaf::new(k, v);

        self.balance()?;

        self._insert(leaf)
    }

    fn _insert(&mut self, leaf: Leaf<K, V>) -> Result<Option<V>, S::Error> {
        let mut old = None;

        match self {
            KelvinMap::Empty => *self = KelvinMap::Leaf(leaf),

            KelvinMap::Leaf(l) if l.key() == leaf.key() => {
                old.replace(l.value().clone());
                *self = KelvinMap::Leaf(leaf);
            }

            KelvinMap::Leaf(l) if l.key() < leaf.key() => {
                let left = Annotated::new(mem::take(self));
                let right = Annotated::new(KelvinMap::Leaf(leaf));

                *self = KelvinMap::Node(left, right);
            }

            KelvinMap::Leaf(l) if leaf.key() < l.key() => {
                let left = Annotated::new(KelvinMap::Leaf(leaf));
                let right = Annotated::new(mem::take(self));

                *self = KelvinMap::Node(left, right);
            }

            KelvinMap::Node(l, _)
                if borrow_max(l.annotation()) >= leaf.key() =>
            {
                old = l.val_mut()?._insert(leaf)?;
            }

            KelvinMap::Node(l, r)
                if borrow_max(l.annotation()) < leaf.key() =>
            {
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
    ///
    /// Internally, a naive balancing will be performed. If the tree contains more elements on the
    /// left, it will move the maximum key of the left to the right - and vice-versa.
    pub fn remove(&mut self, k: &K) -> Result<Option<V>, S::Error> {
        self.balance()?;

        self._remove(k)
    }

    fn _remove(&mut self, k: &K) -> Result<Option<V>, S::Error> {
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

/// Private struct used to hide the complex branch signature behind an
/// `impl Deref<Target = V>` for returning references to values in the map
struct ValRef<'a, K, V, A, S>(Branch<'a, KelvinMap<K, V, A, S>, S>)
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S>,
    S: Store;

impl<'a, K, V, A, S> Deref for ValRef<'a, K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S>,
    S: Store,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &**self.0
    }
}

/// Private struct used to hide the complex branch signature behind an
/// `impl DerefMut<Target = V>` for returning mutable references to values in the map
struct ValRefMut<'a, K, V, A, S>(BranchMut<'a, KelvinMap<K, V, A, S>, S>)
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S> + Borrow<Max<K>>,
    S: Store;

impl<'a, K, V, A, S> Deref for ValRefMut<'a, K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S> + Borrow<Max<K>>,
    S: Store,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &**self.0
    }
}

impl<'a, K, V, A, S> DerefMut for ValRefMut<'a, K, V, A, S>
where
    K: Canon<S> + Ord,
    V: Canon<S>,
    A: Canon<S> + Annotation<KelvinMap<K, V, A, S>, S> + Borrow<Max<K>>,
    S: Store,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut **self.0
    }
}
