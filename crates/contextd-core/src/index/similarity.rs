//! Semantic similarity and token overlap scoring.
//!
//! Provides pure-Rust text similarity computation for event correlation.

use std::collections::HashSet;

/// Computes Jaccard token similarity between two text snippets in [0.0, 1.0].
pub fn compute_jaccard_similarity(query: &str, candidate: &str) -> f64 {
    let tokens_a = tokenize(query);
    let tokens_b = tokenize(candidate);

    if tokens_a.is_empty() && tokens_b.is_empty() {
        return 1.0;
    }
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    let intersection_count = tokens_a.intersection(&tokens_b).count();
    let union_count = tokens_a.union(&tokens_b).count();

    if union_count == 0 {
        0.0
    } else {
        intersection_count as f64 / union_count as f64
    }
}

fn tokenize(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_lowercase())
        .collect()
}
