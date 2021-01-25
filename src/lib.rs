// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(test), no_std)]
#![feature(external_doc)]
#![doc(include = "../README.md")]
#![warn(missing_docs)]

pub use annotation::{MapAnnotation, MapAnnotationDefault};
pub use leaf::Leaf;
pub use map::KelvinMap;

mod annotation;
mod leaf;
mod map;

#[cfg(test)]
mod tests;

/// [`KelvinMap`] default implementation using the minimal [`MapAnnotation`]
pub type Map<K, V, S> = KelvinMap<K, V, MapAnnotationDefault<K, S>, S>;
