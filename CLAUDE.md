# Claude's Notes for waveform-mcp

## Project Overview

An MCP (Model Context Protocol) server for reading and analyzing VCD/FST waveform files using the wellen library.

## Architecture

- **main.rs**: MCP server handler with tool routing, supports stdio and HTTP modes
- **lib.rs**: Library exports from submodules
- **condition.rs/condition.lalrpop**: Parser for conditional expressions (LALRPOP grammar)
- **formatting.rs**: Time and signal value formatting utilities
- **hierarchy.rs**: Signal/scope lookup by hierarchical path
- **signal.rs**: Signal reading, event finding, and metadata

## MCP Tools (7 total)

1. `open_waveform` - Open VCD/FST files, assign optional alias
2. `close_waveform` - Close waveform and free memory
3. `list_signals` - List signals with name_pattern, hierarchy_prefix filters
4. `read_signal` - Read values at time_index or time_indices
5. `get_signal_info` - Get signal type, width, index range
6. `find_signal_events` - Find all changes in time range
7. `find_conditional_events` - Complex condition search with expression parser

## Key Dependencies

- `wellen` - Wavefile parsing
- `rmcp` - MCP server framework
- `tokio` - Async runtime
- `lalrpop` - Condition parser grammar
- `axum` - HTTP server

## Transport Modes

- **Stdio** (default): `cargo run --release`
- **HTTP**: `cargo run --release -- --http --bind-address 127.0.0.1:8000`

## Condition Expression Syntax

Supported in `find_conditional_events`:
- Signal paths: `TOP.signal`
- Bitwise: `~`, `&`, `|`, `^`
- Boolean: `&&`, `||`, `!`
- Comparison: `==`, `!=`
- Special: `$past(signal)` for previous value
- Literals: `4'b0101`, `3'd2`, `5'h1A`
- Bit extract: `signal[bit]` or `signal[msb:lsb]`

Operator precedence (high to low): `~`/`!`, `==`/`!=`, `&`, `^`, `|`, `&&`, `||`

## Build & Test

```bash
cargo build --release
cargo test
```

## Notes for Development

- Waveform store uses `Arc<RwLock<HashMap<>>>` for shared state
- Signal data must be loaded with `waveform.load_signals()` before reading
- Time values formatted with actual timescale (ns, ps, etc.)
- Condition parser grammar in `src/condition/condition.lalrpop`
