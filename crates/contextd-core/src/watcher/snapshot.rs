//! Text diff generation and snapshot comparison.
//!
//! Produces standard unified diff output between two file revisions.

/// Generates a unified diff format string between previous and current text.
pub fn compute_text_diff(
    file_path: &str,
    old_content: &str,
    new_content: &str,
) -> Option<String> {
    if old_content == new_content {
        return None;
    }

    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let clean_path = file_path.strip_prefix('/').unwrap_or(file_path);

    let mut diff = String::new();
    diff.push_str(&format!("--- a/{}\n", clean_path));
    diff.push_str(&format!("+++ b/{}\n", clean_path));

    let mut has_changes = false;
    let max_len = old_lines.len().max(new_lines.len());

    for i in 0..max_len {
        let old_line = old_lines.get(i);
        let new_line = new_lines.get(i);

        match (old_line, new_line) {
            (Some(&o), Some(&n)) if o != n => {
                has_changes = true;
                diff.push_str(&format!("-{}\n", o));
                diff.push_str(&format!("+{}\n", n));
            }
            (Some(&o), None) => {
                has_changes = true;
                diff.push_str(&format!("-{}\n", o));
            }
            (None, Some(&n)) => {
                has_changes = true;
                diff.push_str(&format!("+{}\n", n));
            }
            _ => {}
        }
    }

    if has_changes {
        Some(diff)
    } else {
        None
    }
}
