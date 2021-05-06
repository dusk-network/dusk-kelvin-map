// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(test), no_std)]
#![feature(external_doc)]
#![doc(include = "../README.md")]
#![warn(missing_docs)]
#![feature(ordering_helpers)]

pub use annotation::{MapAnnotation, MapAnnotationDefault};
pub use leaf::Leaf;
pub use map::KelvinMap;

mod annotation;
mod leaf;
mod map;

/// [`KelvinMap`] default implementation using the minimal [`MapAnnotation`]
pub type Map<K, V> = KelvinMap<K, V, MapAnnotationDefault<K>>;
