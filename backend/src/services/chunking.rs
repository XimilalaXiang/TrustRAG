use sha2::{Digest, Sha256};
use text_splitter::MarkdownSplitter;

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub target_chars: usize,
    pub overlap_chars: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            target_chars: 1500,
            overlap_chars: 200,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: usize,
    pub content: String,
    pub content_hash: String,
    pub char_start: usize,
    pub char_end: usize,
    pub heading_path: Option<String>,
}

fn compute_sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract the current heading context from the markdown at a given position.
/// Walks backwards from `pos` to find the most recent heading hierarchy.
fn extract_heading_path(markdown: &str, pos: usize) -> Option<String> {
    let before = &markdown[..pos];
    let mut headings: Vec<(usize, String)> = Vec::new();

    for line in before.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix('#') {
            let level = 1 + stripped.len() - stripped.trim_start_matches('#').len();
            let text = stripped.trim_start_matches('#').trim().to_string();
            if !text.is_empty() && level <= 6 {
                headings.retain(|(lvl, _)| *lvl < level);
                headings.push((level, text));
            }
        }
    }

    if headings.is_empty() {
        None
    } else {
        Some(headings.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join(" > "))
    }
}

pub fn chunk_markdown(markdown: &str, config: &ChunkConfig) -> Vec<TextChunk> {
    let range = config.target_chars.saturating_sub(config.overlap_chars)..config.target_chars;
    let splitter = MarkdownSplitter::new(range);

    let raw_chunks: Vec<&str> = splitter.chunks(markdown).collect();
    let mut result = Vec::with_capacity(raw_chunks.len());

    for (i, chunk_text) in raw_chunks.iter().enumerate() {
        let char_start = markdown.find(chunk_text).unwrap_or(0);
        let char_end = char_start + chunk_text.len();
        let heading_path = extract_heading_path(markdown, char_start);
        let content_hash = compute_sha256(chunk_text);

        result.push(TextChunk {
            index: i,
            content: chunk_text.to_string(),
            content_hash,
            char_start,
            char_end,
            heading_path,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MARKDOWN: &str = r#"# Chapter 1 Introduction

This is the introduction paragraph. It covers the basic concepts and terminology that will be used throughout this document.

## 1.1 Background

Background information about the topic. This section provides historical context and motivation for the work described in this document.

## 1.2 Objectives

The main objectives of this work are:

1. First objective with detailed explanation
2. Second objective with detailed explanation
3. Third objective with detailed explanation

# Chapter 2 Methods

This chapter describes the methods used in our research.

## 2.1 Data Collection

Data was collected from multiple sources including surveys, interviews, and public databases.

## 2.2 Analysis

The analysis was performed using standard statistical methods.
"#;

    #[test]
    fn test_chunk_markdown_produces_chunks() {
        let config = ChunkConfig {
            target_chars: 300,
            overlap_chars: 50,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        assert!(!chunks.is_empty());
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_chunks_cover_all_content() {
        let config = ChunkConfig {
            target_chars: 300,
            overlap_chars: 50,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(TEST_MARKDOWN.contains(&chunk.content));
        }
    }

    #[test]
    fn test_chunks_have_sequential_indices() {
        let config = ChunkConfig {
            target_chars: 300,
            overlap_chars: 50,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_chunks_have_hashes() {
        let config = ChunkConfig {
            target_chars: 300,
            overlap_chars: 50,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        for chunk in &chunks {
            assert!(!chunk.content_hash.is_empty());
            assert_eq!(chunk.content_hash.len(), 64); // SHA-256 hex length
        }
    }

    #[test]
    fn test_chunks_unique_hashes_for_different_content() {
        let config = ChunkConfig {
            target_chars: 300,
            overlap_chars: 50,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        let hashes: std::collections::HashSet<_> =
            chunks.iter().map(|c| &c.content_hash).collect();
        assert_eq!(hashes.len(), chunks.len());
    }

    #[test]
    fn test_chunks_respect_size_limit() {
        let config = ChunkConfig {
            target_chars: 300,
            overlap_chars: 50,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        for chunk in &chunks {
            assert!(
                chunk.content.len() <= config.target_chars + 100,
                "Chunk too large: {} chars",
                chunk.content.len()
            );
        }
    }

    #[test]
    fn test_heading_path_extraction() {
        let md = "# Title\n\n## Section A\n\nText here\n\n### Sub A1\n\nMore text\n\n## Section B\n\nDifferent text";

        let path = extract_heading_path(md, md.find("Text here").unwrap());
        assert_eq!(path.as_deref(), Some("Title > Section A"));

        let path = extract_heading_path(md, md.find("More text").unwrap());
        assert_eq!(path.as_deref(), Some("Title > Section A > Sub A1"));

        let path = extract_heading_path(md, md.find("Different text").unwrap());
        assert_eq!(path.as_deref(), Some("Title > Section B"));
    }

    #[test]
    fn test_chunk_markdown_heading_paths() {
        let config = ChunkConfig {
            target_chars: 200,
            overlap_chars: 30,
        };
        let chunks = chunk_markdown(TEST_MARKDOWN, &config);

        let has_heading_path = chunks.iter().any(|c| c.heading_path.is_some());
        assert!(has_heading_path, "At least one chunk should have a heading path");
    }

    #[test]
    fn test_empty_input() {
        let config = ChunkConfig::default();
        let chunks = chunk_markdown("", &config);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_small_input_single_chunk() {
        let config = ChunkConfig::default();
        let chunks = chunk_markdown("Small text", &config);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Small text");
    }

    #[test]
    fn test_whitespace_only_input() {
        let config = ChunkConfig::default();
        let chunks = chunk_markdown("   \n\n  \t  ", &config);
        assert!(chunks.is_empty(), "Whitespace-only input should produce no chunks");
    }

    #[test]
    fn test_no_content_loss_across_chunks() {
        let md = "# Title\n\nParagraph 1.\n\n## Section\n\nParagraph 2.\n\n### Subsection\n\nParagraph 3.";
        let config = ChunkConfig { target_chars: 40, overlap_chars: 0 };
        let chunks = chunk_markdown(md, &config);
        assert!(!chunks.is_empty());
        let combined: String = chunks.iter().map(|c| c.content.as_str()).collect::<Vec<_>>().join(" ");
        assert!(combined.contains("Paragraph 1"), "Content must not be lost: {combined}");
        assert!(combined.contains("Paragraph 2"), "Content must not be lost: {combined}");
        assert!(combined.contains("Paragraph 3"), "Content must not be lost: {combined}");
    }
}
