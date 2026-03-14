# Token Reduction — Benchmark Results

Token estimates use DISTIRA's BPE approximation (chars ÷ 4).

## Summary

| Intent | Fixtures | Avg raw tokens | Avg compiled tokens | Avg reduction |
|--------|----------|---------------|---------------------|---------------|
| debug  | 2        | 259           | 36                  | **85.1%**     |
| review | 2        | 216           | 37                  | **82.4%**     |
| summarize | 2     | 263           | 56                  | **77.4%**     |
| **combined** | 6 | **246**    | **43**              | **82.5%**     |

## Fixture detail

### debug — `bench_debug_log.jsonl`

| ID | Raw tokens | Compiled tokens | Reduction |
|----|-----------|-----------------|-----------|
| debug-01 | 360 | 42 | 88.3% |
| debug-02 | 158 | 30 | 81.0% |

### review — `bench_git_diff.jsonl`

| ID | Raw tokens | Compiled tokens | Reduction |
|----|-----------|-----------------|-----------|
| diff-01 | 294 | 46 | 84.4% |
| diff-02 | 137 | 28 | 79.6% |

### summarize — `bench_conversation.jsonl`

| ID | Raw tokens | Compiled tokens | Reduction |
|----|-----------|-----------------|-----------|
| conv-01 | 332 | 55 | 83.4% |
| conv-02 | 193 | 57 | 70.5% |

## Notes

- Fixtures are in `fixtures/` as JSONL files (one JSON object per line).
- Token counts are estimated with `chars / 4` (±10% of real BPE counts).
- Reduction includes deduplication, noise removal, and intent-specific trimming by the Context Budget Compiler.
- Memory reuse (Context Memory Lensing) adds additional savings on repeated sessions but is not measured here.
