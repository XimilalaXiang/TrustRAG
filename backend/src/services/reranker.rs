use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::services::search::SearchResult;
use crate::traits::llm_provider::{LlmMessage, LlmProvider, LlmRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReRankConfig {
    pub enabled: bool,
    pub top_n: usize,
    pub method: ReRankMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReRankMethod {
    LlmScoring,
}

impl Default for ReRankConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            top_n: 5,
            method: ReRankMethod::LlmScoring,
        }
    }
}

pub async fn rerank(
    results: Vec<SearchResult>,
    query: &str,
    config: &ReRankConfig,
    llm_provider: &dyn LlmProvider,
) -> Result<Vec<SearchResult>> {
    if !config.enabled || results.is_empty() {
        return Ok(results);
    }

    match config.method {
        ReRankMethod::LlmScoring => llm_rerank(results, query, config.top_n, llm_provider).await,
    }
}

async fn llm_rerank(
    results: Vec<SearchResult>,
    query: &str,
    top_n: usize,
    llm_provider: &dyn LlmProvider,
) -> Result<Vec<SearchResult>> {
    let candidates: Vec<_> = results.iter().take(20).collect();
    if candidates.is_empty() {
        return Ok(results);
    }

    let mut passages = String::new();
    for (i, r) in candidates.iter().enumerate() {
        let snippet = if r.content.len() > 300 {
            &r.content[..300]
        } else {
            &r.content
        };
        passages.push_str(&format!("[{}] {}\n\n", i, snippet.replace('\n', " ")));
    }

    let system_prompt = "You are a relevance ranking assistant. Given a query and passages, \
        rank the passages by relevance to the query. Return ONLY a JSON array of passage \
        indices in descending order of relevance, e.g. [3, 0, 5, 1, 2]. No explanation.";

    let user_prompt = format!(
        "Query: {}\n\nPassages:\n{}\n\nReturn the indices ranked by relevance (most relevant first), as a JSON array:",
        query, passages
    );

    let req = LlmRequest {
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.0,
        max_tokens: 200,
        stream: false,
    };

    let resp = llm_provider.generate(&req).await;

    match resp {
        Ok(response) => {
            let ranked_indices = parse_ranking_response(&response.content, candidates.len());

            let mut reranked: Vec<SearchResult> = Vec::new();
            let mut used = std::collections::HashSet::new();

            for idx in ranked_indices {
                if idx < candidates.len() && !used.contains(&idx) {
                    let mut result = candidates[idx].clone();
                    result.relevance_score = 1.0 - (reranked.len() as f64 / candidates.len() as f64);
                    reranked.push(result);
                    used.insert(idx);
                }
            }

            for (i, r) in candidates.iter().enumerate() {
                if !used.contains(&i) {
                    reranked.push((*r).clone());
                }
            }

            reranked.truncate(top_n);

            tracing::info!(
                query_len = query.len(),
                candidates = candidates.len(),
                reranked = reranked.len(),
                "LLM re-ranking completed"
            );

            Ok(reranked)
        }
        Err(e) => {
            tracing::warn!("LLM re-ranking failed, returning original order: {}", e);
            let mut fallback = results;
            fallback.truncate(top_n);
            Ok(fallback)
        }
    }
}

fn parse_ranking_response(content: &str, max_idx: usize) -> Vec<usize> {
    let trimmed = content.trim();

    let json_str = if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    if let Ok(indices) = serde_json::from_str::<Vec<usize>>(json_str) {
        return indices.into_iter().filter(|i| *i < max_idx).collect();
    }

    let mut indices: Vec<usize> = Vec::new();
    for part in trimmed.split(|c: char| !c.is_ascii_digit()) {
        if let Ok(n) = part.parse::<usize>() {
            if n < max_idx {
                indices.push(n);
            }
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ranking_json_array() {
        let result = parse_ranking_response("[3, 0, 5, 1, 2]", 6);
        assert_eq!(result, vec![3, 0, 5, 1, 2]);
    }

    #[test]
    fn test_parse_ranking_with_text() {
        let result = parse_ranking_response("The ranking is: [2, 0, 1]", 3);
        assert_eq!(result, vec![2, 0, 1]);
    }

    #[test]
    fn test_parse_ranking_filters_out_of_range() {
        let result = parse_ranking_response("[0, 1, 10, 2]", 3);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_parse_ranking_fallback_numbers() {
        let result = parse_ranking_response("3, 1, 0, 2", 4);
        assert_eq!(result, vec![3, 1, 0, 2]);
    }

    #[test]
    fn test_default_config_disabled() {
        let config = ReRankConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.top_n, 5);
    }
}
