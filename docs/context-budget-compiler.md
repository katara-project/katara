# Context Budget Compiler

The compiler transforms raw context into the **smallest useful context**
before the LLM call, reducing cost and improving response quality.

## How it works

1. **Tokenize** the raw input to estimate size.
2. **Classify intent** (debug, summarize, review, general) to choose reduction strategy.
3. **Apply reducers** sequentially to shrink the context while preserving signal.
4. **Auto-select directive template** internally from intent + content signals (transparent to users).
5. **Emit** a `CompileResult` with before/after token counts and a concrete `compiled_context` payload.

## Reduction targets

| Source | Strategy |
| --- | --- |
| Logs | Keep last N lines, deduplicate repeated frames |
| Stack traces | Collapse to top-of-stack + root cause |
| Diffs | Remove unchanged hunks, keep +/- lines |
| Transcripts | Summarize turns, keep latest 2–3 exchanges |
| Conversation history | Rolling window with stable-block reuse |

## API

```rust
pub fn compile_context(raw: &str) -> CompileResult;
pub fn detect_intent(raw: &str) -> String;
```

`CompileResult` now includes:

- `intent`
- `raw_tokens_estimate`
- `compiled_tokens_estimate`
- `summary`
- `compiled_context`

In the chat runtime, DISTIRA uses this `compiled_context` to rewrite the latest user turn before forwarding the request to the routed provider.
The semantic cache also persists this full compiler output so repeated requests can skip recompilation and reuse the same reduced payload.

## Crate

`compiler/` — see [compiler/src/lib.rs](../compiler/src/lib.rs).
