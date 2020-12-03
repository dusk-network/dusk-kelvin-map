[![Repository](https://img.shields.io/badge/github-dusk--kelvin--map-blueviolet)](https://github.com/dusk-network/dusk-kelvin-map)

# Dusk Kelvin Map

Binary search tree implementation with no associated value on the nodes.

It will extend the standard properties of a default BST.

There is a naive balance implementation that will compare the cardinality of the left and right nodes of a tree before inserting. If there is a discrepancy, one minimum/maximum leaf will be swapped, according to the discrepancy. This will progressively balance the tree in every insert operation.

This map-like implementation uses Microkelvin as backend and is optimized to work under constrained/hosted environments such as WASM runtimes.

## Example

```rust
use canonical_host::MemStore;
use dusk_kelvin_map::Map;

// Create a new map u64 -> u32 that will use MemStore as storage backend
let mut map: Map<u64, u32, MemStore> = Map::default();

// Insert a new mapping 2 -> 4
map.insert(2, 4).expect("Failed to insert data.");

// Fetch the key 2 and expect it to be 4
let value = map
    .get(&2)
    .expect("Error traversing the map.")
    .expect("No valid leaf was found for the provided key.");
assert_eq!(4, value);

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
assert_eq!(5, new);

// Mutate the key 2 to the previous value + 2, and return the new value + 1
// Since the previous value is 5, the new value should be 7, and the returned mapping should be 8
let map_mutated = map
    .map_mut(&2, |v| {
        *v += 2;
        *v + 1
    })
    .expect("Error executing a mutable map over the value.")
    .expect("No valid leaf was found for the provided key.");
let value = map
    .get(&2)
    .expect("Error traversing the map.")
    .expect("No valid leaf was found for the provided key.");
assert_eq!(8, map_mutated);
assert_eq!(7, value);

// Remove the key 2
let value = map
    .remove(&2)
    .expect("Error traversing the map.")
    .expect("No valid leaf was found for the provided key.");
assert_eq!(7, value);
```
