#![forbid(unsafe_code)]

//! Seeded-Model-Programming (SMP) foundation for balanced ternary systems.
//!
//! A "seed" is a compact determiner of inference behavior: a small data structure
//! that encodes how a model should respond to inputs. Seeds can be encoded from
//! observed behavior, decoded to reconstruct behavior, mutated for variation,
//! combined for new capabilities, and stored in a SeedBank.

use std::collections::HashMap;

/// A single balanced ternary value: -1, 0, or +1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    pub fn value(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }
}

/// A compact inference determiner. Encodes a mapping from input indices to
/// ternary outputs, representing the behavioral signature of a model role.
#[derive(Clone, Debug, PartialEq)]
pub struct Seed {
    pub id: u64,
    pub entries: Vec<(u64, Trit)>,
    pub generation: u32,
}

impl Seed {
    pub fn new(id: u64) -> Self {
        Seed { id, entries: Vec::new(), generation: 0 }
    }

    pub fn with_entries(id: u64, entries: Vec<(u64, Trit)>) -> Self {
        Seed { id, entries, generation: 0 }
    }

    pub fn lookup(&self, input_hash: u64) -> Option<Trit> {
        for (k, v) in &self.entries {
            if *k == input_hash {
                return Some(*v);
            }
        }
        None
    }

    pub fn insert(&mut self, input_hash: u64, trit: Trit) {
        for (k, v) in &mut self.entries {
            if *k == input_hash {
                *v = trit;
                return;
            }
        }
        self.entries.push((input_hash, trit));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Encodes observed behavior into a Seed.
pub struct SeedEncoder;

impl SeedEncoder {
    pub fn new() -> Self {
        SeedEncoder
    }

    /// Create a seed from a slice of (input_hash, output_trit) observations.
    pub fn encode(&self, id: u64, observations: &[(u64, Trit)]) -> Seed {
        let mut seed = Seed::new(id);
        for &(input_hash, trit) in observations {
            seed.insert(input_hash, trit);
        }
        seed
    }

    /// Encode from raw i8 pairs, filtering out invalid trits.
    pub fn encode_from_i8(&self, id: u64, pairs: &[(u64, i8)]) -> Seed {
        let observations: Vec<(u64, Trit)> = pairs
            .iter()
            .filter_map(|&(h, v)| Trit::from_i8(v).map(|t| (h, t)))
            .collect();
        self.encode(id, &observations)
    }
}

impl Default for SeedEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decodes a Seed back into a behavioral mapping.
pub struct SeedDecoder;

impl SeedDecoder {
    pub fn new() -> Self {
        SeedDecoder
    }

    /// Decode seed into a hashmap of input_hash -> trit.
    pub fn decode(&self, seed: &Seed) -> HashMap<u64, Trit> {
        seed.entries.iter().map(|&(k, v)| (k, v)).collect()
    }

    /// Decode a single lookup from the seed.
    pub fn decode_one(&self, seed: &Seed, input_hash: u64) -> Option<Trit> {
        seed.lookup(input_hash)
    }

    /// Decode all outputs as i8 values.
    pub fn decode_as_i8(&self, seed: &Seed) -> Vec<(u64, i8)> {
        seed.entries.iter().map(|&(k, v)| (k, v.value())).collect()
    }
}

impl Default for SeedDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutates a seed with controlled variation.
pub struct SeedMutator {
    pub flip_probability: f64, // 0.0 to 1.0
    pub generation_budget: u32,
}

impl SeedMutator {
    pub fn new(flip_probability: f64) -> Self {
        SeedMutator { flip_probability, generation_budget: 100 }
    }

    /// Deterministic single-step mutation: flip entry at given index if threshold met.
    /// Uses simple modular arithmetic as a pseudo-random stand-in (no external deps).
    pub fn mutate_entry(&self, seed: &mut Seed, index: usize, threshold: u64) -> bool {
        if index >= seed.entries.len() {
            return false;
        }
        // Use threshold as pseudo-random: if threshold % 100 < flip_probability * 100, flip
        let should_flip = ((threshold % 100) as f64) < self.flip_probability * 100.0;
        if should_flip {
            let (_, ref mut trit) = seed.entries[index];
            *trit = match *trit {
                Trit::Neg => Trit::Zero,
                Trit::Zero => Trit::Pos,
                Trit::Pos => Trit::Neg,
            };
            seed.generation += 1;
        }
        should_flip
    }

    /// Mutate all entries with deterministic "randomness" derived from a base counter.
    pub fn mutate_all(&self, seed: &mut Seed, base_counter: u64) -> usize {
        let len = seed.entries.len();
        let mut flipped = 0;
        for i in 0..len {
            let threshold = base_counter.wrapping_add(i as u64 * 37);
            if self.mutate_entry(seed, i, threshold) {
                flipped += 1;
            }
        }
        flipped
    }
}

/// Combines two seeds to produce a new seed with merged capabilities.
pub struct SeedCombiner;

impl SeedCombiner {
    pub fn new() -> Self {
        SeedCombiner
    }

    /// Combine two seeds using a ternary majority rule for overlapping keys,
    /// and include all non-overlapping entries.
    pub fn combine(&self, seed_a: &Seed, seed_b: &Seed, new_id: u64) -> Seed {
        let mut combined = Seed::new(new_id);
        combined.generation = seed_a.generation.max(seed_b.generation) + 1;

        // Add all from seed_a
        for &(k, v) in &seed_a.entries {
            combined.insert(k, v);
        }

        // Merge from seed_b: for overlaps, use majority with zero as tiebreak
        for &(k, v_b) in &seed_b.entries {
            if let Some(v_a) = combined.lookup(k) {
                let combined_val = match (v_a, v_b) {
                    (Trit::Pos, Trit::Pos) => Trit::Pos,
                    (Trit::Neg, Trit::Neg) => Trit::Neg,
                    (Trit::Zero, _) | (_, Trit::Zero) => Trit::Zero,
                    (Trit::Pos, Trit::Neg) | (Trit::Neg, Trit::Pos) => Trit::Zero,
                };
                combined.insert(k, combined_val);
            } else {
                combined.insert(k, v_b);
            }
        }

        combined
    }
}

impl Default for SeedCombiner {
    fn default() -> Self {
        Self::new()
    }
}

/// A storage and retrieval system for seeds.
#[derive(Clone, Debug)]
pub struct SeedBank {
    seeds: HashMap<u64, Seed>,
    name: String,
}

impl SeedBank {
    pub fn new(name: &str) -> Self {
        SeedBank { seeds: HashMap::new(), name: name.to_string() }
    }

    pub fn store(&mut self, seed: Seed) {
        self.seeds.insert(seed.id, seed);
    }

    pub fn retrieve(&self, id: u64) -> Option<&Seed> {
        self.seeds.get(&id)
    }

    pub fn remove(&mut self, id: u64) -> Option<Seed> {
        self.seeds.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.seeds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seeds.is_empty()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn all_ids(&self) -> Vec<u64> {
        self.seeds.keys().copied().collect()
    }

    /// Find seeds that contain a given input_hash mapping.
    pub fn find_by_input(&self, input_hash: u64) -> Vec<&Seed> {
        self.seeds.values().filter(|s| s.lookup(input_hash).is_some()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trit_roundtrip() {
        assert_eq!(Trit::from_i8(Trit::Neg.value()).unwrap(), Trit::Neg);
        assert_eq!(Trit::from_i8(Trit::Zero.value()).unwrap(), Trit::Zero);
        assert_eq!(Trit::from_i8(Trit::Pos.value()).unwrap(), Trit::Pos);
    }

    #[test]
    fn seed_new_empty() {
        let seed = Seed::new(42);
        assert_eq!(seed.id, 42);
        assert!(seed.is_empty());
        assert_eq!(seed.generation, 0);
    }

    #[test]
    fn seed_insert_and_lookup() {
        let mut seed = Seed::new(1);
        seed.insert(100, Trit::Pos);
        seed.insert(200, Trit::Neg);
        assert_eq!(seed.lookup(100), Some(Trit::Pos));
        assert_eq!(seed.lookup(200), Some(Trit::Neg));
        assert_eq!(seed.lookup(999), None);
        assert_eq!(seed.len(), 2);
    }

    #[test]
    fn seed_insert_overwrite() {
        let mut seed = Seed::new(1);
        seed.insert(100, Trit::Pos);
        seed.insert(100, Trit::Neg);
        assert_eq!(seed.lookup(100), Some(Trit::Neg));
        assert_eq!(seed.len(), 1);
    }

    #[test]
    fn seed_encoder_basic() {
        let encoder = SeedEncoder::new();
        let seed = encoder.encode(1, &[(10, Trit::Pos), (20, Trit::Neg)]);
        assert_eq!(seed.lookup(10), Some(Trit::Pos));
        assert_eq!(seed.lookup(20), Some(Trit::Neg));
    }

    #[test]
    fn seed_encoder_from_i8() {
        let encoder = SeedEncoder::new();
        let seed = encoder.encode_from_i8(1, &[(10, -1), (20, 0), (30, 1), (40, 5)]);
        assert_eq!(seed.len(), 3); // (40, 5) is invalid, filtered out
        assert_eq!(seed.lookup(10), Some(Trit::Neg));
        assert_eq!(seed.lookup(20), Some(Trit::Zero));
        assert_eq!(seed.lookup(30), Some(Trit::Pos));
    }

    #[test]
    fn seed_decoder_to_map() {
        let decoder = SeedDecoder::new();
        let seed = Seed::with_entries(1, vec![(1, Trit::Pos), (2, Trit::Neg)]);
        let map = decoder.decode(&seed);
        assert_eq!(map.len(), 2);
        assert_eq!(map[&1], Trit::Pos);
        assert_eq!(map[&2], Trit::Neg);
    }

    #[test]
    fn seed_decoder_single() {
        let decoder = SeedDecoder::new();
        let seed = Seed::with_entries(1, vec![(42, Trit::Zero)]);
        assert_eq!(decoder.decode_one(&seed, 42), Some(Trit::Zero));
        assert_eq!(decoder.decode_one(&seed, 99), None);
    }

    #[test]
    fn seed_decoder_as_i8() {
        let decoder = SeedDecoder::new();
        let seed = Seed::with_entries(1, vec![(1, Trit::Neg), (2, Trit::Zero), (3, Trit::Pos)]);
        let pairs = decoder.decode_as_i8(&seed);
        assert!(pairs.contains(&(1, -1)));
        assert!(pairs.contains(&(2, 0)));
        assert!(pairs.contains(&(3, 1)));
    }

    #[test]
    fn seed_mutator_flip() {
        let mutator = SeedMutator::new(1.0); // always flip
        let mut seed = Seed::with_entries(1, vec![(10, Trit::Pos)]);
        let flipped = mutator.mutate_entry(&mut seed, 0, 50);
        assert!(flipped);
        assert_eq!(seed.lookup(10), Some(Trit::Neg));
        assert_eq!(seed.generation, 1);
    }

    #[test]
    fn seed_mutator_no_flip() {
        let mutator = SeedMutator::new(0.0); // never flip
        let mut seed = Seed::with_entries(1, vec![(10, Trit::Pos)]);
        let flipped = mutator.mutate_entry(&mut seed, 0, 50);
        assert!(!flipped);
        assert_eq!(seed.lookup(10), Some(Trit::Pos));
    }

    #[test]
    fn seed_mutator_out_of_bounds() {
        let mutator = SeedMutator::new(1.0);
        let mut seed = Seed::new(1);
        let flipped = mutator.mutate_entry(&mut seed, 0, 50);
        assert!(!flipped);
    }

    #[test]
    fn seed_mutator_mutate_all() {
        let mutator = SeedMutator::new(1.0);
        let mut seed = Seed::with_entries(1, vec![(10, Trit::Pos), (20, Trit::Neg), (30, Trit::Zero)]);
        let flipped = mutator.mutate_all(&mut seed, 0);
        assert_eq!(flipped, 3);
        assert_eq!(seed.lookup(10), Some(Trit::Neg));
        assert_eq!(seed.lookup(20), Some(Trit::Zero));
        assert_eq!(seed.lookup(30), Some(Trit::Pos));
    }

    #[test]
    fn seed_combiner_no_overlap() {
        let combiner = SeedCombiner::new();
        let a = Seed::with_entries(1, vec![(10, Trit::Pos)]);
        let b = Seed::with_entries(2, vec![(20, Trit::Neg)]);
        let combined = combiner.combine(&a, &b, 3);
        assert_eq!(combined.len(), 2);
        assert_eq!(combined.lookup(10), Some(Trit::Pos));
        assert_eq!(combined.lookup(20), Some(Trit::Neg));
    }

    #[test]
    fn seed_combiner_overlap_agree() {
        let combiner = SeedCombiner::new();
        let a = Seed::with_entries(1, vec![(10, Trit::Pos)]);
        let b = Seed::with_entries(2, vec![(10, Trit::Pos)]);
        let combined = combiner.combine(&a, &b, 3);
        assert_eq!(combined.lookup(10), Some(Trit::Pos));
    }

    #[test]
    fn seed_combiner_overlap_disagree() {
        let combiner = SeedCombiner::new();
        let a = Seed::with_entries(1, vec![(10, Trit::Pos)]);
        let b = Seed::with_entries(2, vec![(10, Trit::Neg)]);
        let combined = combiner.combine(&a, &b, 3);
        assert_eq!(combined.lookup(10), Some(Trit::Zero)); // tiebreak
    }

    #[test]
    fn seed_combiner_generation() {
        let combiner = SeedCombiner::new();
        let mut a = Seed::new(1);
        a.generation = 5;
        let mut b = Seed::new(2);
        b.generation = 3;
        let combined = combiner.combine(&a, &b, 3);
        assert_eq!(combined.generation, 6);
    }

    #[test]
    fn seed_bank_store_retrieve() {
        let mut bank = SeedBank::new("test");
        bank.store(Seed::with_entries(1, vec![(10, Trit::Pos)]));
        assert_eq!(bank.len(), 1);
        assert_eq!(bank.retrieve(1).unwrap().lookup(10), Some(Trit::Pos));
    }

    #[test]
    fn seed_bank_remove() {
        let mut bank = SeedBank::new("test");
        bank.store(Seed::new(1));
        let removed = bank.remove(1);
        assert!(removed.is_some());
        assert!(bank.is_empty());
    }

    #[test]
    fn seed_bank_find_by_input() {
        let mut bank = SeedBank::new("test");
        bank.store(Seed::with_entries(1, vec![(42, Trit::Pos)]));
        bank.store(Seed::with_entries(2, vec![(42, Trit::Neg)]));
        bank.store(Seed::with_entries(3, vec![(99, Trit::Zero)]));
        let found = bank.find_by_input(42);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn seed_bank_all_ids() {
        let mut bank = SeedBank::new("test");
        bank.store(Seed::new(10));
        bank.store(Seed::new(20));
        let mut ids = bank.all_ids();
        ids.sort();
        assert_eq!(ids, vec![10, 20]);
    }
}
