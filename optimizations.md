# Optimizations

## Measurements

| Version    | `synth/subsynth_plain` | `synth/subsynth_with_containers` |
| ---------- | ---------------------- | -------------------------------- |
| prerelease | 22ms                   | 40ms                             |
| 0.1.0      | 5.34ms                 | 9.22ms                           |
| 0.1.1      | 4.37ms                 | 7.93ms                           |
| 0.1.2      | 4.75ms                 | 8.80ms                           |
| 0.2.1      | 3.73ms                 | 5.76ms                           |
| 0.3.0      | 6.01ms                 | 8.73ms                           |
| 0.3.1      | 4.24ms                 | 6.59ms                           |
| 0.3.3      | 4.29ms                 | 4.51ms                           |

## Changelogs

### 0.1.0
Used to check if a node was already calculated in the graph using a boolean. Changed to a generational reference to avoid having to reset nodes after calling `next_sample`. ControlNode no longer uses HashMaps. Now stores node data in the weights of each node on the graph.

### 0.1.1
When interpreting `Constant` nodes, set the generation to `u64::MAX`, which prevents constants from being needlessly re-evaluated.

### 0.1.2
Switched from a `DiGraph` to a `StableDiGraph` to avoid invalidating node indices across removals.

### 0.2.1
Stores the order that nodes should be processed in a cache instead of manually traversing the graph every time.

### 0.3.0
Upgraded to stereo samples (previously mono). Benchmarks demonstrate that "plain" processes have very little overhead. Containers still need optimizing. Using an enum with Mono and Stereo variants in an attempt to avoid calculating mono signals twice was much slower, since we would be branching at very high rates when performing match statements.

### 0.3.1
Avoided reevaluating constants (again). Now that we're caching node traversal, we remove the previous method which did this, because graph traversal only triggers when the graph is modified, so constants should be re-evaluated in case they have been changed. The simple fix was to skip adding Constant nodes to the cache.

### 0.3.3
Avoids evaluating container input/output nodes after caching.