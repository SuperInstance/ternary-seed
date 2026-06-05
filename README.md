# ternary-seed: Seeded-Model-Programming (SMP) foundation for {-1, 0, +1} inference

## Why This Exists

In a system where the same base model takes on different roles (analyst, critic, generator, evaluator), you need a compact way to encode and reproduce those behavioral differences. A "seed" captures the inference fingerprint of a model role — what it outputs for given inputs — in a small, composable, mutable structure. This is the foundation for Seeded-Model-Programming: creating different model behaviors from the same base by varying the seed.

## Core Concepts

**Balanced ternary**: Three values: -1 (Neg), 0 (Zero), +1 (Pos). The domain of inference outputs.

**Seed**: A compact mapping from input hashes to ternary outputs. Think of it as a behavioral fingerprint — a lookup table that determines what a model outputs for given inputs.

**SeedEncoder**: Observes (input, output) pairs and packs them into a Seed.

**SeedDecoder**: Reconstructs a behavioral mapping from a Seed, producing a hashmap of expected outputs.

**SeedMutator**: Applies controlled variation to a Seed's entries, cycling Neg→Zero→Pos→Neg. Uses deterministic thresholds instead of random numbers for reproducibility.

**SeedCombiner**: Merges two Seeds. Overlapping entries use majority rule; conflicting entries (Pos vs Neg) resolve to Zero.

**SeedBank**: A named storage for Seeds with lookup by ID or by input hash.

## Quick Start

```toml
[dependencies]
ternary-seed = "0.1"
```

```rust
use ternary_seed::*;

let encoder = SeedEncoder::new();
let seed = encoder.encode(1, &[(100, Trit::Pos), (200, Trit::Neg), (300, Trit::Zero)]);

let decoder = SeedDecoder::new();
assert_eq!(decoder.decode_one(&seed, 100), Some(Trit::Pos));

let mut bank = SeedBank::new("models");
bank.store(seed);
assert_eq!(bank.len(), 1);
```

## API Overview

| Type | Description |
|------|-------------|
| `Trit` | Balanced ternary value: Neg, Zero, or Pos |
| `Seed` | Compact mapping from input hashes to ternary outputs |
| `SeedEncoder` | Creates Seeds from observed (input, output) pairs |
| `SeedDecoder` | Reconstructs behavior from a Seed |
| `SeedMutator` | Applies controlled variation to Seed entries |
| `SeedCombiner` | Merges two Seeds using majority rule for conflicts |
| `SeedBank` | Named storage for Seeds with retrieval by ID or input hash |

## How It Works

A Seed stores entries as a flat Vec of (u64, Trit) pairs — no hashing beyond what the user provides for input keys. Lookups are linear scan, which is fine for the typical seed sizes (tens to hundreds of entries).

Mutation uses a deterministic threshold comparison: the user provides a `flip_probability` (0.0–1.0) and a counter-derived threshold. If `threshold % 100 < flip_probability * 100`, the entry cycles to its next value. This gives reproducible "randomness" without external RNG dependencies.

Combination uses ternary majority: agreeing entries keep their value, Pos vs Neg conflicts resolve to Zero, and anything involving Zero tends toward Zero. The generation counter increments to track combinatorial lineage.

## Known Limitations

- Linear-scan lookup in Seeds. Performance degrades with large seeds (thousands of entries). Not suitable for real-time lookup in hot paths.
- Mutation "randomness" is deterministic from a counter — not cryptographically random, not even statistically uniform. Fine for controlled variation, bad for unbiased sampling.
- No collision handling: if two different inputs hash to the same u64 key, only the latest entry survives.
- Seeds have no size limit. A SeedBank with many large Seeds consumes unbounded memory.
- No serialization. Seeds exist only in memory.

## Use Cases

- **Model role differentiation**: Encode different inference behaviors (cautious, aggressive, neutral) as Seeds applied to the same base model.
- **Behavioral A/B testing**: Mutate a Seed slightly, compare the two behaviors, keep the better one.
- **Ensemble seeding**: Combine Seeds from multiple specialized models to create a generalist behavior.
- **Evolutionary search**: Mutate and combine Seeds over generations to optimize for a target behavior.

## Ecosystem Context

Foundation crate for the SuperInstance SMP (Seeded-Model-Programming) layer. Upstream of `ternary-evolution-advanced` and `ternary-fitness` (which would evaluate seed quality). Could use `ternary-hash` for input hashing. Related to `ternary-inference` for the actual model inference that Seeds parameterize.

## License

MIT

## See Also
- **ternary-genome** — related
- **ternary-ga** — related
- **ternary-fitness** — related
- **ternary-random** — related
- **ternary-evolution-advanced** — related

