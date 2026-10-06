use std::collections::{BTreeMap, BTreeSet};

/// |A ∩ B| / |A ∪ B|
pub fn jaccard<T: Ord>(a: &BTreeSet<T>, b: &BTreeSet<T>) -> f64 {
    let inter = a.intersection(b).count();
    let union = a.len() + b.len() - inter;
    if union == 0 {
        return 0.0;
    }
    inter as f64 / union as f64
}

/// Weighted Jaccard: sum of weights of A ∩ B over sum of weights of A ∪ B.
pub fn weighted_jaccard<T: Ord>(a: &BTreeSet<T>, b: &BTreeSet<T>, w: impl Fn(&T) -> f64) -> f64 {
    let (mut inter, mut union) = (0.0, 0.0);
    for x in a {
        let wx = w(x);
        union += wx;
        if b.contains(x) {
            inter += wx;
        }
    }
    for x in b {
        if !a.contains(x) {
            union += w(x);
        }
    }
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

/// |A ∩ B| / min(|A|, |B|): high when one unit is (nearly) contained in the other.
pub fn containment<T: Ord>(a: &BTreeSet<T>, b: &BTreeSet<T>) -> f64 {
    let m = a.len().min(b.len());
    if m == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / m as f64
}

/// Cosine similarity of two count histograms.
pub fn cosine(a: &BTreeMap<String, u32>, b: &BTreeMap<String, u32>) -> f64 {
    let dot: f64 = a
        .iter()
        .filter_map(|(k, x)| b.get(k).map(|y| *x as f64 * *y as f64))
        .sum();
    let na: f64 = a.values().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

/// Dice coefficient of two multisets: 2 * sum(min counts) / (|A| + |B|).
pub fn multiset_dice<K: Ord>(a: &BTreeMap<K, u32>, b: &BTreeMap<K, u32>) -> f64 {
    let total: u32 = a.values().sum::<u32>() + b.values().sum::<u32>();
    if total == 0 {
        return 0.0;
    }
    let inter: u32 = a
        .iter()
        .filter_map(|(k, x)| b.get(k).map(|y| *x.min(y)))
        .sum();
    2.0 * inter as f64 / total as f64
}

/// 2 * LCS(a, b) / (|a| + |b|): order-aware sequence similarity.
pub fn lcs_ratio<T: PartialEq>(a: &[T], b: &[T]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let mut prev = vec![0usize; b.len() + 1];
    for x in a {
        let mut cur = vec![0usize; b.len() + 1];
        for (j, y) in b.iter().enumerate() {
            cur[j + 1] = if x == y {
                prev[j] + 1
            } else {
                cur[j].max(prev[j + 1])
            };
        }
        prev = cur;
    }
    2.0 * prev[b.len()] as f64 / (a.len() + b.len()) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaccard_and_containment() {
        let a: BTreeSet<u32> = [1, 2, 3, 4].into();
        let b: BTreeSet<u32> = [3, 4].into();
        assert_eq!(jaccard(&a, &b), 0.5);
        assert_eq!(containment(&a, &b), 1.0);
        assert_eq!(jaccard(&BTreeSet::<u32>::new(), &BTreeSet::new()), 0.0);
        let m1: BTreeMap<u8, u32> = [(1, 2), (2, 1)].into();
        let m2: BTreeMap<u8, u32> = [(1, 1), (3, 1)].into();
        assert_eq!(multiset_dice(&m1, &m2), 0.4);
        assert_eq!(lcs_ratio(&[1, 2, 3], &[1, 3]), 0.8);
    }
}
