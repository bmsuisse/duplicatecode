use std::collections::BTreeSet;

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
