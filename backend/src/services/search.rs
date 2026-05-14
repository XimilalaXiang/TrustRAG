use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::traits::embedding_provider::EmbeddingProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub mode: SearchMode,
    pub top_k: usize,
    pub min_score: f64,
    pub use_mmr: bool,
    pub mmr_lambda: f64,
    pub rrf_k: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Vector,
    Fulltext,
    Hybrid,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            mode: SearchMode::Hybrid,
            top_k: 10,
            min_score: 0.3,
            use_mmr: false,
            mmr_lambda: 0.7,
            rrf_k: 60.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SearchResult {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub heading_path: Option<String>,
    pub page_start: Option<i32>,
    pub page_end: Option<i32>,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub search_time_ms: u64,
}

/// Vector search using pgvector cosine similarity.
pub async fn vector_search(
    pool: &PgPool,
    workspace_id: Uuid,
    query_embedding: &[f32],
    top_k: usize,
    document_ids: Option<&[Uuid]>,
) -> anyhow::Result<Vec<(Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)>> {
    let embedding_str = format!(
        "[{}]",
        query_embedding
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let rows = if let Some(doc_ids) = document_ids {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)>(
            r#"
            SELECT dc.id, dc.document_id, dc.content, dc.heading_path,
                   dc.page_start, dc.page_end,
                   1.0 - (dc.embedding <=> $1::vector) as score
            FROM document_chunks dc
            JOIN documents d ON dc.document_id = d.id
            WHERE d.workspace_id = $2
              AND dc.embedding IS NOT NULL
              AND dc.document_id = ANY($3)
            ORDER BY dc.embedding <=> $1::vector
            LIMIT $4
            "#,
        )
        .bind(&embedding_str)
        .bind(workspace_id)
        .bind(doc_ids)
        .bind(top_k as i64)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)>(
            r#"
            SELECT dc.id, dc.document_id, dc.content, dc.heading_path,
                   dc.page_start, dc.page_end,
                   1.0 - (dc.embedding <=> $1::vector) as score
            FROM document_chunks dc
            JOIN documents d ON dc.document_id = d.id
            WHERE d.workspace_id = $2
              AND dc.embedding IS NOT NULL
            ORDER BY dc.embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        .bind(&embedding_str)
        .bind(workspace_id)
        .bind(top_k as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(rows)
}

/// Full-text search using pg_trgm similarity.
pub async fn fulltext_search(
    pool: &PgPool,
    workspace_id: Uuid,
    query: &str,
    top_k: usize,
    document_ids: Option<&[Uuid]>,
) -> anyhow::Result<Vec<(Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)>> {
    let rows = if let Some(doc_ids) = document_ids {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)>(
            r#"
            SELECT dc.id, dc.document_id, dc.content, dc.heading_path,
                   dc.page_start, dc.page_end,
                   similarity(dc.content, $1)::float8 as score
            FROM document_chunks dc
            JOIN documents d ON dc.document_id = d.id
            WHERE d.workspace_id = $2
              AND dc.document_id = ANY($3)
              AND dc.content % $1
            ORDER BY similarity(dc.content, $1) DESC
            LIMIT $4
            "#,
        )
        .bind(query)
        .bind(workspace_id)
        .bind(doc_ids)
        .bind(top_k as i64)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)>(
            r#"
            SELECT dc.id, dc.document_id, dc.content, dc.heading_path,
                   dc.page_start, dc.page_end,
                   similarity(dc.content, $1)::float8 as score
            FROM document_chunks dc
            JOIN documents d ON dc.document_id = d.id
            WHERE d.workspace_id = $2
              AND dc.content % $1
            ORDER BY similarity(dc.content, $1) DESC
            LIMIT $3
            "#,
        )
        .bind(query)
        .bind(workspace_id)
        .bind(top_k as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(rows)
}

/// Reciprocal Rank Fusion (RRF) to merge two ranked lists.
pub fn rrf_fusion(
    vector_results: &[(Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)],
    fulltext_results: &[(Uuid, Uuid, String, Option<String>, Option<i32>, Option<i32>, f64)],
    k: f64,
    top_k: usize,
) -> Vec<SearchResult> {
    use std::collections::HashMap;

    let mut scores: HashMap<Uuid, (f64, Uuid, String, Option<String>, Option<i32>, Option<i32>)> = HashMap::new();

    for (rank, row) in vector_results.iter().enumerate() {
        let rrf_score = 1.0 / (k + rank as f64 + 1.0);
        let entry = scores.entry(row.0).or_insert((0.0, row.1, row.2.clone(), row.3.clone(), row.4, row.5));
        entry.0 += rrf_score;
    }

    for (rank, row) in fulltext_results.iter().enumerate() {
        let rrf_score = 1.0 / (k + rank as f64 + 1.0);
        let entry = scores.entry(row.0).or_insert((0.0, row.1, row.2.clone(), row.3.clone(), row.4, row.5));
        entry.0 += rrf_score;
    }

    let mut results: Vec<SearchResult> = scores
        .into_iter()
        .map(|(chunk_id, (score, doc_id, content, heading, page_start, page_end))| SearchResult {
            chunk_id,
            document_id: doc_id,
            content,
            heading_path: heading,
            page_start,
            page_end,
            relevance_score: score,
        })
        .collect();

    results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(top_k);
    results
}

/// Execute a hybrid search (vector + fulltext + RRF).
pub async fn hybrid_search(
    pool: &PgPool,
    embedding_provider: &dyn EmbeddingProvider,
    workspace_id: Uuid,
    query: &str,
    config: &SearchConfig,
    document_ids: Option<&[Uuid]>,
) -> anyhow::Result<SearchResponse> {
    let start = std::time::Instant::now();
    let retrieval_k = config.top_k * 2;

    let results = match config.mode {
        SearchMode::Vector => {
            let embeddings = embedding_provider.embed_texts(&[query.to_string()]).await?;
            let query_emb = embeddings.into_iter().next().ok_or_else(|| anyhow::anyhow!("No embedding returned"))?;
            let vector_rows = vector_search(pool, workspace_id, &query_emb, config.top_k, document_ids).await?;
            vector_rows
                .into_iter()
                .map(|r| SearchResult {
                    chunk_id: r.0,
                    document_id: r.1,
                    content: r.2,
                    heading_path: r.3,
                    page_start: r.4,
                    page_end: r.5,
                    relevance_score: r.6,
                })
                .collect()
        }
        SearchMode::Fulltext => {
            let ft_rows = fulltext_search(pool, workspace_id, query, config.top_k, document_ids).await?;
            ft_rows
                .into_iter()
                .map(|r| SearchResult {
                    chunk_id: r.0,
                    document_id: r.1,
                    content: r.2,
                    heading_path: r.3,
                    page_start: r.4,
                    page_end: r.5,
                    relevance_score: r.6,
                })
                .collect()
        }
        SearchMode::Hybrid => {
            let embeddings = embedding_provider.embed_texts(&[query.to_string()]).await?;
            let query_emb = embeddings.into_iter().next().ok_or_else(|| anyhow::anyhow!("No embedding returned"))?;

            let (vector_rows, ft_rows) = tokio::try_join!(
                vector_search(pool, workspace_id, &query_emb, retrieval_k, document_ids),
                fulltext_search(pool, workspace_id, query, retrieval_k, document_ids),
            )?;

            rrf_fusion(&vector_rows, &ft_rows, config.rrf_k, config.top_k)
        }
    };

    let effective_min_score = if config.mode == SearchMode::Hybrid {
        0.0
    } else {
        config.min_score
    };

    let filtered: Vec<SearchResult> = results
        .into_iter()
        .filter(|r| r.relevance_score >= effective_min_score)
        .collect();

    let total = filtered.len();
    let elapsed = start.elapsed().as_millis() as u64;

    tracing::info!(
        mode = ?config.mode,
        query_len = query.len(),
        pre_filter_count = total,
        elapsed_ms = elapsed,
        "Search completed"
    );

    Ok(SearchResponse {
        results: filtered,
        total,
        search_time_ms: elapsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_basic() {
        let vector_results = vec![
            (Uuid::new_v4(), Uuid::new_v4(), "chunk A".into(), Some("heading".into()), Some(1), Some(1), 0.9),
            (Uuid::new_v4(), Uuid::new_v4(), "chunk B".into(), None, Some(2), Some(2), 0.8),
        ];
        let fulltext_results = vec![
            (vector_results[1].0, vector_results[1].1, "chunk B".into(), None, Some(2), Some(2), 0.7),
            (Uuid::new_v4(), Uuid::new_v4(), "chunk C".into(), None, Some(3), Some(3), 0.6),
        ];

        let results = rrf_fusion(&vector_results, &fulltext_results, 60.0, 10);

        assert_eq!(results.len(), 3);
        let b_result = results.iter().find(|r| r.content == "chunk B").unwrap();
        let a_result = results.iter().find(|r| r.content == "chunk A").unwrap();
        assert!(b_result.relevance_score > a_result.relevance_score,
            "chunk B should rank higher due to appearing in both lists");
    }

    #[test]
    fn test_rrf_fusion_top_k_limit() {
        let vector_results: Vec<_> = (0..5)
            .map(|i| (Uuid::new_v4(), Uuid::new_v4(), format!("v{}", i), None, None, None, 0.9 - i as f64 * 0.1))
            .collect();
        let fulltext_results: Vec<_> = (0..5)
            .map(|i| (Uuid::new_v4(), Uuid::new_v4(), format!("f{}", i), None, None, None, 0.9 - i as f64 * 0.1))
            .collect();

        let results = rrf_fusion(&vector_results, &fulltext_results, 60.0, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.mode, SearchMode::Hybrid);
        assert_eq!(config.top_k, 10);
    }

    #[test]
    fn test_rrf_fusion_empty_inputs() {
        let results = rrf_fusion(&[], &[], 60.0, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_fusion_one_side_empty() {
        let vector_results = vec![
            (Uuid::new_v4(), Uuid::new_v4(), "only in vector".into(), None, None, None, 0.9),
        ];
        let results = rrf_fusion(&vector_results, &[], 60.0, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "only in vector");
        let expected_score = 1.0 / (60.0 + 0.0 + 1.0);
        assert!((results[0].relevance_score - expected_score).abs() < 1e-10);
    }

    #[test]
    fn test_rrf_fusion_score_math_correctness() {
        let id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();
        let vector_results = vec![
            (id, doc_id, "shared".into(), None, None, None, 0.9),
        ];
        let fulltext_results = vec![
            (id, doc_id, "shared".into(), None, None, None, 0.8),
        ];

        let results = rrf_fusion(&vector_results, &fulltext_results, 60.0, 10);
        assert_eq!(results.len(), 1);
        let expected = 1.0 / 61.0 + 1.0 / 61.0;
        assert!((results[0].relevance_score - expected).abs() < 1e-10,
            "Score should be sum of both RRF contributions: expected {expected}, got {}",
            results[0].relevance_score);
    }

    #[test]
    fn test_rrf_fusion_ranking_stability() {
        let ids: Vec<_> = (0..5).map(|_| Uuid::new_v4()).collect();
        let doc_id = Uuid::new_v4();

        let vector_results: Vec<_> = ids.iter()
            .enumerate()
            .map(|(i, &id)| (id, doc_id, format!("chunk_{}", i), None, None, None, 0.0))
            .collect();

        let results = rrf_fusion(&vector_results, &[], 60.0, 5);
        for i in 0..results.len() - 1 {
            assert!(results[i].relevance_score >= results[i + 1].relevance_score,
                "Results should be sorted by descending score");
        }
    }
}
