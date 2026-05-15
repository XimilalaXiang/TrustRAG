use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewRecord {
    pub id: Uuid,
    pub citation_id: Uuid,
    pub reviewer_id: Uuid,
    pub status: String,
    pub comment: Option<String>,
    pub corrected_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewInput {
    pub status: String,
    pub comment: Option<String>,
    pub corrected_text: Option<String>,
}

pub async fn create_review(
    pool: &DbPool,
    citation_id: Uuid,
    reviewer_id: Uuid,
    input: &CreateReviewInput,
) -> anyhow::Result<ReviewRecord> {
    let valid_statuses = ["approved", "rejected", "flagged", "pending"];
    if !valid_statuses.contains(&input.status.as_str()) {
        anyhow::bail!("Invalid review status: {}", input.status);
    }

    let record = sqlx::query_as::<_, ReviewRecord>(
        r#"
        INSERT INTO review_records (citation_id, reviewer_id, status, comment, corrected_text)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, citation_id, reviewer_id, status, comment, corrected_text, created_at, updated_at
        "#,
    )
    .bind(citation_id.to_string())
    .bind(reviewer_id)
    .bind(&input.status)
    .bind(&input.comment)
    .bind(&input.corrected_text)
    .fetch_one(pool)
    .await?;

    if input.status == "approved" {
        sqlx::query("UPDATE citations SET verified = true WHERE id = $1")
            .bind(citation_id.to_string())
            .execute(pool)
            .await?;
    } else if input.status == "rejected" || input.status == "flagged" {
        sqlx::query("UPDATE citations SET verified = false WHERE id = $1")
            .bind(citation_id.to_string())
            .execute(pool)
            .await?;
    }

    Ok(record)
}

pub async fn list_reviews_for_citation(
    pool: &DbPool,
    citation_id: Uuid,
) -> anyhow::Result<Vec<ReviewRecord>> {
    let records = sqlx::query_as::<_, ReviewRecord>(
        r#"
        SELECT id, citation_id, reviewer_id, status, comment, corrected_text, created_at, updated_at
        FROM review_records
        WHERE citation_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(citation_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(records)
}

#[derive(Debug, Serialize)]
pub struct ReviewStats {
    pub total_citations: i64,
    pub approved: i64,
    pub rejected: i64,
    pub flagged: i64,
    pub pending: i64,
    pub unreviewed: i64,
}

pub async fn get_review_stats_for_conversation(
    pool: &DbPool,
    conversation_id: Uuid,
) -> anyhow::Result<ReviewStats> {
    let total_citations = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM citations c
        JOIN messages m ON c.message_id = m.id
        WHERE m.conversation_id = $1
        "#,
    )
    .bind(conversation_id.to_string())
    .fetch_one(pool)
    .await?;

    let reviewed = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT latest.status, COUNT(*) FROM (
            SELECT DISTINCT ON (rr.citation_id) rr.status
            FROM review_records rr
            JOIN citations c ON rr.citation_id = c.id
            JOIN messages m ON c.message_id = m.id
            WHERE m.conversation_id = $1
            ORDER BY rr.citation_id, rr.created_at DESC
        ) latest
        GROUP BY latest.status
        "#,
    )
    .bind(conversation_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut approved = 0i64;
    let mut rejected = 0i64;
    let mut flagged = 0i64;
    let mut pending_count = 0i64;

    for (status, count) in &reviewed {
        match status.as_str() {
            "approved" => approved = *count,
            "rejected" => rejected = *count,
            "flagged" => flagged = *count,
            "pending" => pending_count = *count,
            _ => {}
        }
    }

    let reviewed_total = approved + rejected + flagged + pending_count;
    let unreviewed = total_citations - reviewed_total;

    Ok(ReviewStats {
        total_citations,
        approved,
        rejected,
        flagged,
        pending: pending_count,
        unreviewed: unreviewed.max(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_review_input_deserialize() {
        let json = r#"{"status":"approved","comment":"Looks correct"}"#;
        let input: CreateReviewInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.status, "approved");
        assert_eq!(input.comment, Some("Looks correct".to_string()));
        assert_eq!(input.corrected_text, None);
    }

    #[test]
    fn test_create_review_input_all_fields() {
        let json = r#"{"status":"rejected","comment":"Wrong source","corrected_text":"Fixed text"}"#;
        let input: CreateReviewInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.status, "rejected");
        assert_eq!(input.comment, Some("Wrong source".to_string()));
        assert_eq!(input.corrected_text, Some("Fixed text".to_string()));
    }

    #[test]
    fn test_create_review_input_minimal() {
        let json = r#"{"status":"flagged"}"#;
        let input: CreateReviewInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.status, "flagged");
        assert_eq!(input.comment, None);
        assert_eq!(input.corrected_text, None);
    }
}
