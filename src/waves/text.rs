//! Text preparation and full-document pooling for discussion retrieval.

/// Keep original text in the lexical index; remove repeated reply quotations and
/// conventional signatures only from semantic input so replies describe their own topic.
pub(super) fn semantic_body(body: &str, is_mail: bool) -> String {
    if !is_mail {
        return body.trim().to_owned();
    }
    let mut lines = Vec::new();
    for line in body.lines() {
        if line == "-- " || line.trim() == "_______________________________________________" {
            break;
        }
        if line.trim_start().starts_with('>') {
            continue;
        }
        // Attribution lines have little semantic value without the quoted reply.
        if line.trim_end().ends_with("wrote:") {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n").trim().to_owned()
}

/// Pool all passages, including the end of long messages, instead of allowing the
/// model to truncate a whole-message input at its sequence limit.
pub(super) fn pooled_embedding(vectors: &[Vec<f32>]) -> Vec<f32> {
    let mut mean = vec![0.0_f32; 384];
    for vector in vectors {
        for (sum, value) in mean.iter_mut().zip(vector) {
            *sum += value;
        }
    }
    let norm = mean.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut mean {
            *value /= norm;
        }
    }
    mean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotations_and_signatures_do_not_dominate_reply_vectors() {
        let body =
            "On Tuesday Alice wrote:\n> unrelated topic\n\nMy answer about scope.\n-- \nSignature";
        assert_eq!(semantic_body(body, true), "My answer about scope.");
        assert_eq!(semantic_body(body, false), body.trim());
    }

    #[test]
    fn pooling_includes_late_passages_and_normalizes() {
        let mut early = vec![0.0; 384];
        let mut late = vec![0.0; 384];
        early[0] = 1.0;
        late[1] = 1.0;
        let pooled = pooled_embedding(&[early, late]);
        assert!((pooled[0] - pooled[1]).abs() < 1e-6);
        assert!((pooled.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-6);
    }
}
