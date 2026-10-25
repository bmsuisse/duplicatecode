use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Sorted, distinct hashes of every k-gram of the normalized token stream.
pub fn kgram_hashes(tokens: &[String], k: usize) -> Vec<u64> {
    let mut out = Vec::new();
    if tokens.len() < k {
        if !tokens.is_empty() {
            out.push(hash_slice(tokens));
        }
        return out;
    }
    out.extend(tokens.windows(k).map(hash_slice));
    out.sort_unstable();
    out.dedup();
    out
}

fn hash_slice(w: &[String]) -> u64 {
    let mut h = DefaultHasher::new();
    w.hash(&mut h);
    h.finish()
}
