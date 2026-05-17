use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalParseResult {
    pub markdown: String,
    pub metadata: LocalDocMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalDocMetadata {
    pub title: Option<String>,
    pub page_count: Option<i32>,
    pub language: Option<String>,
}

pub fn parse_local(data: &[u8], filename: &str, file_type: &str) -> anyhow::Result<LocalParseResult> {
    match file_type {
        "txt" | "md" => parse_text(data, filename),
        "html" => parse_html(data, filename),
        "pdf" => parse_pdf_fallback(data, filename),
        "docx" => parse_docx_fallback(data, filename),
        _ => anyhow::bail!("Unsupported file type for local processing: {}", file_type),
    }
}

fn parse_text(data: &[u8], filename: &str) -> anyhow::Result<LocalParseResult> {
    let text = String::from_utf8_lossy(data).to_string();
    let title = filename
        .rsplit('/')
        .next()
        .and_then(|f| f.rsplit('.').last())
        .unwrap_or(filename)
        .to_string();

    Ok(LocalParseResult {
        markdown: text,
        metadata: LocalDocMetadata {
            title: Some(title),
            page_count: Some(1),
            language: None,
        },
    })
}

fn parse_html(data: &[u8], filename: &str) -> anyhow::Result<LocalParseResult> {
    let html = String::from_utf8_lossy(data);

    let re_tags = regex::Regex::new(r"<[^>]+>")?;
    let re_spaces = regex::Regex::new(r"\s+")?;

    let text = re_tags.replace_all(&html, " ");
    let text = re_spaces.replace_all(&text, " ").trim().to_string();

    let title_re = regex::Regex::new(r"<title>([^<]+)</title>")?;
    let title = title_re
        .captures(&html)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| filename.to_string());

    Ok(LocalParseResult {
        markdown: text,
        metadata: LocalDocMetadata {
            title: Some(title),
            page_count: Some(1),
            language: None,
        },
    })
}

fn parse_pdf_fallback(data: &[u8], filename: &str) -> anyhow::Result<LocalParseResult> {
    #[cfg(sqlite_mode)]
    {
        let _ = data;
        Ok(LocalParseResult {
            markdown: format!(
                "# {}\n\n*PDF parsing requires the doc-processor service. \
                 Please install it or convert this PDF to text/markdown format first.*\n\n\
                 File size: {} bytes",
                filename,
                data.len()
            ),
            metadata: LocalDocMetadata {
                title: Some(filename.to_string()),
                page_count: None,
                language: None,
            },
        })
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = (data, filename);
        anyhow::bail!("PDF processing not available in this build")
    }
}

fn parse_docx_fallback(data: &[u8], filename: &str) -> anyhow::Result<LocalParseResult> {
    #[cfg(sqlite_mode)]
    {
        let _ = data;
        Ok(LocalParseResult {
            markdown: format!(
                "# {}\n\n*DOCX parsing requires the doc-processor service. \
                 Please install it or convert this DOCX to text/markdown format first.*\n\n\
                 File size: {} bytes",
                filename,
                data.len()
            ),
            metadata: LocalDocMetadata {
                title: Some(filename.to_string()),
                page_count: None,
                language: None,
            },
        })
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = (data, filename);
        anyhow::bail!("DOCX processing not available in this build")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let data = b"Hello, world!\nThis is a test document.";
        let result = parse_local(data, "test.txt", "txt").unwrap();
        assert!(result.markdown.contains("Hello, world!"));
        assert_eq!(result.metadata.page_count, Some(1));
    }

    #[test]
    fn test_parse_markdown() {
        let data = b"# Title\n\nSome content here.";
        let result = parse_local(data, "readme.md", "md").unwrap();
        assert!(result.markdown.contains("# Title"));
    }

    #[test]
    fn test_parse_html() {
        let data = b"<html><head><title>Test Page</title></head><body><h1>Hello</h1><p>World</p></body></html>";
        let result = parse_local(data, "page.html", "html").unwrap();
        assert!(result.markdown.contains("Hello"));
        assert!(result.markdown.contains("World"));
        assert_eq!(result.metadata.title, Some("Test Page".to_string()));
    }

    #[test]
    fn test_unsupported_type() {
        let result = parse_local(b"data", "file.xyz", "xyz");
        assert!(result.is_err());
    }
}
