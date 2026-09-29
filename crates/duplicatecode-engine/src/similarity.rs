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
    let dot: f64 = a.iter().filter_map(|(k, x)| b.get(k).map(|y| *x as f64 * *y as f64)).sum();
    let na: f64 = a.values().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
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
    }
}
