//! Unit QA tests for Jaccard token similarity calculation.

#[cfg(test)]
mod tests {
    use contextd_core::index::compute_jaccard_similarity;

    #[test]
    fn test_identical_strings_return_one() {
        let text = "oom-killer invoked on pid 12345 in slice user.slice";
        let score = compute_jaccard_similarity(text, text);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_completely_disjoint_strings_return_zero() {
        let a = "systemd journal rotated successfully";
        let b = "quantum computing breakthrough reported";
        let score = compute_jaccard_similarity(a, b);
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_partial_token_overlap() {
        let a = "nginx failed to bind address 0.0.0.0:80";
        let b = "apache failed to bind address 0.0.0.0:80";
        let score = compute_jaccard_similarity(a, b);
        // Shared tokens: failed, to, bind, address, 0.0.0.0, 80 (approx 6 tokens)
        // Disjoint: nginx, apache
        assert!(score > 0.5 && score < 1.0);
    }

    #[test]
    fn test_empty_string_handling() {
        assert_eq!(compute_jaccard_similarity("", ""), 1.0);
        assert_eq!(compute_jaccard_similarity("hello", ""), 0.0);
        assert_eq!(compute_jaccard_similarity("", "world"), 0.0);
    }
}
