[![Repository](https://img.shields.io/badge/github-dusk--kelvin--map-blueviolet)](https://github.com/dusk-network/dusk-kelvin-map)
[![Docs](https://img.shields.io/badge/docs--rs-dusk--kelvin--map-blue)](https://docs.rs/dusk-kelvin-map)

# Dusk Kelvin Map

#### NOTE: This crate is deprecated in favor of [dusk-hamt](https://crates.io/crates/dusk-hamt)

Binary search tree implementation with no associated value on the nodes.

It will extend the standard properties of a default BST.

There is a naive balance implementation that will compare the cardinality of the left and right nodes of a tree before any mutation of the map (insert / remove). If there is a discrepancy, one minimum/maximum leaf will be swapped, according to the discrepancy. This will progressively balance the tree.

This implementation uses Microkelvin as backend and is optimized to work under constrained/hosted environments such as WASM runtimes.

## Example

```rust
use dusk_kelvin_map::Map;

// Create a new map u64 -> u32 that will use MemStore as storage backend.
let mut map: Map<u64, u32> = Map::default();

// Insert a new mapping 2 -> 4
map.insert(2, 4).expect("Failed to insert data.");

// Fetch the key 2 and expect it to be 4
let value = map
    .get(&2)
    .expect("Error traversing the map.")
    .expect("No valid leaf was found for the provided key.");
assert_eq!(4, *value);

drop(value);

// Replace the key 2 with 5 via `insert` method
let old = map
    .insert(2, 5)
    .expect("Failed to insert data.")
    .expect("The key was previously inserted and now should be returned as replacement.");
		
let new = map
    .get(&2)
    .expect("Error traversing the map.")
    .expect("No valid leaf was found for the provided key.");
		
assert_eq!(4, old);
assert_eq!(5, *new);

drop(new);

// Remove the key 2
let value = map
    .remove(&2)
    .expect("Error traversing the map.")
    .expect("No valid leaf was found for the provided key.");
```
