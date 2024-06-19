# Grid Project

## Functionality
Most of the things that the Bitwig grid can do, plus a "recorder" node with a controllable playhead

## Performance
- Two different execution modes
	1. Parallel and Immutable on the GPU via SPIR-V. Should "run ahead"
	2. Sequential on the CPU. Start with interpreter, then follow with x86 dynarec

## Problems to solve
- Prevent cyclic connections
	- See https://docs.rs/daggy/latest/daggy/struct.WouldCycle.html
- Support buffered nodes that cycle back to previous nodes
	- Represented as an input (into the buffer) and output (out of the buffer) nodes.
	- Buffers should by circular if we're looking for sample-accuracy
	- Prevents parallelism for any connected nodes
