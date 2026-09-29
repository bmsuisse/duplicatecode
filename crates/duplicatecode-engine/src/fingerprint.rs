use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

/// Hashes of every k-gram of the normalized token stream.
pub fn kgram_hashes(tokens: &[String], k: usize) -> BTreeSet<u64> {
    let mut out = BTreeSet::new();
    if tokens.len() < k {
        if !tokens.is_empty() {
            out.insert(hash_slice(tokens));
        }
        return out;
    }
    for w in tokens.windows(k) {
        out.insert(hash_slice(w));
    }
    out
}

fn hash_slice(w: &[String]) -> u64 {
    let mut h = DefaultHasher::new();
    w.hash(&mut h);
    h.finish()
}
