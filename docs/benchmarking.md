# Benchmarking

Benchmarks use **fixture-based runs** with known inputs and expected outputs.
This ensures reproducible measurements before the full runtime is implemented.

## Token-reduction fixture

| Metric | Value |
| --- | --- |
| Raw context tokens | 2 100 |
| Compiled context tokens | 760 |
| Memory reused tokens | 540 |
| Token avoidance ratio | 63.8 % |

See [benchmarks/token-reduction/results.md](../benchmarks/token-reduction/results.md).

## Request-latency fixture

Placeholder for future provider runtime latency tests.
See [benchmarks/request-latency/results.md](../benchmarks/request-latency/results.md).

## Methodology

1. Prepare a raw context payload of known size.
2. Run through the compiler and memory lensing pipeline.
3. Record `raw_tokens`, `compiled_tokens`, `reused_tokens`.
4. Compute `token_avoidance_ratio = (raw - compiled) / raw`.
5. Compare against baseline to track regression.
