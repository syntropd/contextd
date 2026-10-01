//! Semantic indexing and hybrid search for Linux systemd journal records.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Ingested Linux journald log record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalRecord {
    pub message: String,
    pub unit: Option<String>,
    pub priority: Option<i32>,
    pub timestamp_us: Option<u64>,
    pub embedding: Option<Vec<f32>>,
}

/// Ranked match result for a semantic journal query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalMatch {
    pub message: String,
    pub unit: Option<String>,
    pub priority: Option<i32>,
    pub score: f64,
    pub timestamp_us: Option<u64>,
}

/// Hybrid BM25 and vector semantic embedding index for journal records.
#[derive(Debug, Default, Clone)]
pub struct JournalSemanticIndex {
    records: Vec<JournalRecord>,
    doc_tokens: Vec<Vec<String>>,
    doc_freq: HashMap<String, usize>,
    total_tokens: usize,
}

impl JournalSemanticIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests a structured journal record into the index.
    pub fn ingest_record(&mut self, record: JournalRecord) {
        let tokens = tokenize(&record.message);
        self.total_tokens += tokens.len();

        let unique_terms: HashSet<String> = tokens.iter().cloned().collect();
        for term in unique_terms {
            *self.doc_freq.entry(term).or_insert(0) += 1;
        }

        self.doc_tokens.push(tokens);
        self.records.push(record);
    }

    /// Ingests newline-delimited journalctl JSON records (`journalctl -o json`).
    pub fn ingest_json_lines(&mut self, json_lines: &str) -> usize {
        let mut count = 0;
        for line in json_lines.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(record) = parse_journal_value(&val) {
                    self.ingest_record(record);
                    count += 1;
                }
            }
        }
        count
    }

    /// Queries the index using hybrid BM25 and token similarity.
    pub fn search(&self, query: &str, limit: usize) -> Vec<JournalMatch> {
        self.search_internal(query, None, limit)
    }

    /// Queries the index with precomputed query embedding.
    pub fn search_with_embedding(&self, query: &str, query_embedding: &[f32], limit: usize) -> Vec<JournalMatch> {
        self.search_internal(query, Some(query_embedding), limit)
    }

    fn search_internal(&self, query: &str, embedding: Option<&[f32]>, limit: usize) -> Vec<JournalMatch> {
        if self.records.is_empty() {
            return Vec::new();
        }

        let query_tokens = tokenize(query);
        let n_docs = self.records.len() as f64;
        let avg_dl = (self.total_tokens as f64 / n_docs).max(1.0);
        let k1 = 1.2;
        let b = 0.75;

        let mut scored: Vec<(f64, usize)> = self.records.iter().enumerate().map(|(idx, rec)| {
            let doc = &self.doc_tokens[idx];
            let doc_len = doc.len() as f64;

            let mut bm25 = 0.0;
            let mut tf_map = HashMap::new();
            for t in doc {
                *tf_map.entry(t.as_str()).or_insert(0usize) += 1;
            }

            for q in &query_tokens {
                let tf = *tf_map.get(q.as_str()).unwrap_or(&0) as f64;
                if tf > 0.0 {
                    let df = *self.doc_freq.get(q).unwrap_or(&1) as f64;
                    let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln().max(0.1);
                    let term_score = idf * (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * (doc_len / avg_dl)));
                    bm25 += term_score;
                }
            }

            let mut final_score = bm25;
            if let (Some(q_emb), Some(doc_emb)) = (embedding, &rec.embedding) {
                let cosine = cosine_similarity(q_emb, doc_emb);
                final_score = 0.5 * bm25 + 0.5 * (cosine as f64 * 10.0);
            }

            (final_score, idx)
        }).collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter()
            .take(limit)
            .filter(|(s, _)| *s > 0.0)
            .map(|(score, idx)| {
                let rec = &self.records[idx];
                JournalMatch {
                    message: rec.message.clone(),
                    unit: rec.unit.clone(),
                    priority: rec.priority,
                    score,
                    timestamp_us: rec.timestamp_us,
                }
            })
            .collect()
    }
}

fn parse_journal_value(val: &serde_json::Value) -> Option<JournalRecord> {
    let message = match val.get("MESSAGE") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => {
            let bytes: Vec<u8> = arr.iter().filter_map(|v| v.as_u64().map(|b| b as u8)).collect();
            String::from_utf8_lossy(&bytes).to_string()
        }
        _ => return None,
    };

    let unit = val.get("_SYSTEMD_UNIT").and_then(|v| v.as_str()).map(ToString::to_string);
    let priority = val.get("PRIORITY").and_then(|v| {
        v.as_i64().map(|n| n as i32).or_else(|| v.as_str().and_then(|s| s.parse::<i32>().ok()))
    });
    let timestamp_us = val.get("__REALTIME_TIMESTAMP").and_then(|v| {
        v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
    });

    Some(JournalRecord {
        message,
        unit,
        priority,
        timestamp_us,
        embedding: None,
    })
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_lowercase())
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_and_bm25_semantic_search() {
        let mut idx = JournalSemanticIndex::new();
        let lines = r#"{"MESSAGE":"Out of memory: Kill process 1234 (python3)","PRIORITY":"3","_SYSTEMD_UNIT":"app.service","__REALTIME_TIMESTAMP":"1727800000000000"}
{"MESSAGE":"Starting OpenSSH server daemon...","PRIORITY":"6","_SYSTEMD_UNIT":"sshd.service","__REALTIME_TIMESTAMP":"1727800001000000"}"#;

        assert_eq!(idx.ingest_json_lines(lines), 2);
        let matches = idx.search("memory kill", 10);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].message.contains("Out of memory"));
        assert_eq!(matches[0].unit.as_deref(), Some("app.service"));
        assert_eq!(matches[0].priority, Some(3));
    }

    #[test]
    fn test_cosine_similarity_computation() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-5);
        assert!(cosine_similarity(&v1, &v3).abs() < 1e-5);
    }
}
