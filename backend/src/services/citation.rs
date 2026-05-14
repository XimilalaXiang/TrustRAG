use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::rag::AssembledSource;

/// Parsed citation reference from LLM output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRef {
    pub index: usize,
    pub position: usize,
}

/// Extracted and verified citation ready for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedCitation {
    pub citation_index: usize,
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub quoted_text: String,
    pub page_number: Option<i32>,
    pub heading_path: Option<String>,
    pub relevance_score: f64,
    pub verification: VerificationResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationResult {
    Valid,
    IndexOutOfRange,
}

/// Extract citation markers [1], [2], etc. from LLM response text
pub fn extract_citations(text: &str) -> Vec<CitationRef> {
    let re = Regex::new(r"\[(\d+)\]").unwrap();
    let mut refs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in re.captures_iter(text) {
        if let Ok(index) = cap[1].parse::<usize>() {
            if index > 0 && seen.insert(index) {
                let pos = cap.get(0).unwrap().start();
                refs.push(CitationRef {
                    index,
                    position: pos,
                });
            }
        }
    }

    refs.sort_by_key(|r| r.index);
    refs
}

/// Match extracted citation references against assembled sources
pub fn verify_citations(
    citation_refs: &[CitationRef],
    sources: &[AssembledSource],
) -> Vec<ExtractedCitation> {
    let mut results = Vec::new();

    for cref in citation_refs {
        if let Some(source) = sources.iter().find(|s| s.index == cref.index) {
            results.push(ExtractedCitation {
                citation_index: cref.index,
                chunk_id: source.chunk_id,
                document_id: source.document_id,
                quoted_text: source.content.chars().take(300).collect(),
                page_number: source.page_start,
                heading_path: source.heading_path.clone(),
                relevance_score: source.score,
                verification: VerificationResult::Valid,
            });
        } else {
            results.push(ExtractedCitation {
                citation_index: cref.index,
                chunk_id: Uuid::nil(),
                document_id: Uuid::nil(),
                quoted_text: String::new(),
                page_number: None,
                heading_path: None,
                relevance_score: 0.0,
                verification: VerificationResult::IndexOutOfRange,
            });
        }
    }

    results
}

/// Store verified citations in the database
pub async fn store_citations(
    pool: &PgPool,
    message_id: Uuid,
    citations: &[ExtractedCitation],
) -> anyhow::Result<Vec<Uuid>> {
    let mut ids = Vec::new();

    for c in citations {
        if c.verification != VerificationResult::Valid {
            continue;
        }

        let id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO citations
                (message_id, document_id, chunk_id, citation_index, quoted_text,
                 page_number, heading_path, relevance_score, verified)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)
             RETURNING id",
        )
        .bind(message_id)
        .bind(c.document_id)
        .bind(c.chunk_id)
        .bind(c.citation_index as i16)
        .bind(&c.quoted_text)
        .bind(c.page_number)
        .bind(&c.heading_path)
        .bind(c.relevance_score as f32)
        .fetch_one(pool)
        .await?;

        ids.push(id);
    }

    Ok(ids)
}

/// Full citation pipeline: extract → verify → store
pub async fn process_citations(
    pool: &PgPool,
    message_id: Uuid,
    response_text: &str,
    sources: &[AssembledSource],
) -> anyhow::Result<Vec<ExtractedCitation>> {
    let refs = extract_citations(response_text);
    let citations = verify_citations(&refs, sources);
    store_citations(pool, message_id, &citations).await?;
    Ok(citations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_citations() {
        let text = "根据安全规范 [1]，项目需要满足要求 [2]。此外 [3] 也提到了相关内容。";
        let refs = extract_citations(text);
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].index, 1);
        assert_eq!(refs[1].index, 2);
        assert_eq!(refs[2].index, 3);
    }

    #[test]
    fn test_extract_duplicate_citations() {
        let text = "根据 [1] 的描述，以及 [2] 的补充。再次引用 [1] 的内容。";
        let refs = extract_citations(text);
        assert_eq!(refs.len(), 2); // [1] appears twice but only counted once
    }

    #[test]
    fn test_extract_no_citations() {
        let text = "这是一个没有引用的回答。";
        let refs = extract_citations(text);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_zero_index_ignored() {
        let text = "引用 [0] 应该被忽略，但 [1] 不应该。";
        let refs = extract_citations(text);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].index, 1);
    }

    #[test]
    fn test_verify_valid_citations() {
        let refs = vec![
            CitationRef { index: 1, position: 10 },
            CitationRef { index: 2, position: 30 },
        ];

        let sources = vec![
            AssembledSource {
                index: 1,
                chunk_id: Uuid::new_v4(),
                document_id: Uuid::new_v4(),
                heading_path: Some("Ch1".into()),
                page_start: Some(5),
                page_end: Some(5),
                content: "Source 1 content".into(),
                score: 0.95,
            },
            AssembledSource {
                index: 2,
                chunk_id: Uuid::new_v4(),
                document_id: Uuid::new_v4(),
                heading_path: None,
                page_start: None,
                page_end: None,
                content: "Source 2 content".into(),
                score: 0.85,
            },
        ];

        let results = verify_citations(&refs, &sources);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].verification, VerificationResult::Valid);
        assert_eq!(results[1].verification, VerificationResult::Valid);
        assert_eq!(results[0].page_number, Some(5));
    }

    #[test]
    fn test_verify_out_of_range_citation() {
        let refs = vec![
            CitationRef { index: 1, position: 10 },
            CitationRef { index: 5, position: 30 }, // No source 5
        ];

        let sources = vec![
            AssembledSource {
                index: 1,
                chunk_id: Uuid::new_v4(),
                document_id: Uuid::new_v4(),
                heading_path: None,
                page_start: None,
                page_end: None,
                content: "Source 1".into(),
                score: 0.9,
            },
        ];

        let results = verify_citations(&refs, &sources);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].verification, VerificationResult::Valid);
        assert_eq!(results[1].verification, VerificationResult::IndexOutOfRange);
    }

    #[test]
    fn test_extract_large_indices() {
        let text = "参考 [10] 和 [25] 的内容。";
        let refs = extract_citations(text);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].index, 10);
        assert_eq!(refs[1].index, 25);
    }

    #[test]
    fn test_citation_position_tracking() {
        let text = "Hello [1] world [2]";
        let refs = extract_citations(text);
        assert_eq!(refs[0].position, 6); // "[1]" starts at position 6
    }
}
