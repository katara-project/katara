# Context Memory Lensing

KATARA decomposes recurring context into **stable memory blocks**
and sends only deltas when possible, avoiding redundant token spend.

## How it works

1. Each incoming context is split into blocks identified by content hash.
2. Blocks with high **stability** (unchanged across requests) are cached.
3. On subsequent requests, only **delta tokens** (new or changed blocks) are sent.
4. The **context reuse ratio** quantifies the savings.

## Key metrics

| Metric | Description |
| --- | --- |
| `reused_tokens` | Tokens from stable blocks not re-sent |
| `delta_tokens` | Tokens that changed since last request |
| `context_reuse_ratio` | `reused_tokens / total_tokens` |

## API

```rust
pub fn summarize_memory(raw_tokens: usize) -> MemorySummary;
```

## Crate

`memory/` — see [memory/src/lib.rs](../memory/src/lib.rs).
