import re
from io import BytesIO

import fitz  # PyMuPDF

from app.models import (
    DocumentMetadata,
    HeadingInfo,
    PageData,
    ParseResult,
    TextBlock,
)

HEADING_FONT_SIZE_THRESHOLD = 14.0
SUB_HEADING_FONT_SIZE_THRESHOLD = 12.0

HEADING_PATTERNS = [
    re.compile(r"^第[一二三四五六七八九十百千万\d]+[章节篇]"),
    re.compile(r"^[一二三四五六七八九十]+[、.]"),
    re.compile(r"^\d+(\.\d+)*[.\s]"),
    re.compile(r"^(Chapter|Section|Part)\s+\d+", re.IGNORECASE),
    re.compile(r"^(Abstract|Introduction|Conclusion|References|Appendix)", re.IGNORECASE),
]


def detect_heading_level(text: str, font_size: float, is_bold: bool, page_font_sizes: list[float]) -> int | None:
    """Heuristic heading detection based on font size, bold, and text patterns."""
    text = text.strip()
    if not text or len(text) > 200:
        return None

    avg_font = sum(page_font_sizes) / len(page_font_sizes) if page_font_sizes else 12.0

    if font_size >= avg_font * 1.5 or font_size >= HEADING_FONT_SIZE_THRESHOLD:
        if is_bold:
            return 1
        return 2

    if (font_size >= avg_font * 1.2 or font_size >= SUB_HEADING_FONT_SIZE_THRESHOLD) and is_bold:
        return 2

    if is_bold and font_size >= avg_font:
        for pattern in HEADING_PATTERNS:
            if pattern.match(text):
                return 3

    for pattern in HEADING_PATTERNS[:2]:
        if pattern.match(text):
            if font_size > avg_font:
                return 2
            return 3

    return None


def extract_page_blocks(page: fitz.Page) -> tuple[list[TextBlock], list[float]]:
    """Extract text blocks with font info from a single page."""
    blocks = []
    font_sizes = []

    text_dict = page.get_text("dict", flags=fitz.TEXT_PRESERVE_WHITESPACE)

    for block in text_dict.get("blocks", []):
        if block["type"] != 0:  # skip image blocks
            continue

        block_text_parts = []
        block_font_size = 0.0
        block_is_bold = False
        total_chars = 0

        for line in block.get("lines", []):
            line_text = ""
            for span in line.get("spans", []):
                span_text = span["text"]
                if not span_text.strip():
                    continue
                line_text += span_text
                char_count = len(span_text)
                total_chars += char_count
                block_font_size += span["size"] * char_count
                if "bold" in span.get("font", "").lower() or (span.get("flags", 0) & 2**4):
                    block_is_bold = True
                font_sizes.append(span["size"])
            if line_text.strip():
                block_text_parts.append(line_text.strip())

        content = " ".join(block_text_parts).strip()
        if not content:
            continue

        avg_font = block_font_size / total_chars if total_chars > 0 else 12.0
        bbox = block["bbox"]  # [x0, y0, x1, y1]

        heading_level = detect_heading_level(content, avg_font, block_is_bold, font_sizes)

        blocks.append(TextBlock(
            type="heading" if heading_level else "text",
            content=content,
            bbox=list(bbox),
            heading_level=heading_level,
            font_size=round(avg_font, 1),
            is_bold=block_is_bold,
        ))

    return blocks, font_sizes


def blocks_to_markdown(pages: list[PageData]) -> str:
    """Convert extracted page blocks to Markdown."""
    md_parts = []

    for page in pages:
        for block in page.blocks:
            if block.type == "heading" and block.heading_level:
                prefix = "#" * block.heading_level
                md_parts.append(f"\n{prefix} {block.content}\n")
            else:
                md_parts.append(f"\n{block.content}\n")

    return "\n".join(md_parts).strip()


def extract_headings(pages: list[PageData]) -> list[HeadingInfo]:
    """Collect all headings from parsed pages."""
    headings = []
    for page in pages:
        for i, block in enumerate(page.blocks):
            if block.type == "heading" and block.heading_level:
                headings.append(HeadingInfo(
                    text=block.content,
                    level=block.heading_level,
                    page=page.page_number,
                    block_index=i,
                ))
    return headings


def parse_pdf(file_bytes: bytes) -> ParseResult:
    """Parse a PDF file and return structured result with Markdown and position mapping."""
    doc = fitz.open(stream=BytesIO(file_bytes), filetype="pdf")

    meta = doc.metadata or {}
    pages: list[PageData] = []
    all_font_sizes: list[float] = []

    for page_num in range(len(doc)):
        page = doc[page_num]
        blocks, font_sizes = extract_page_blocks(page)
        all_font_sizes.extend(font_sizes)

        page_text = page.get_text("text").strip()
        rect = page.rect

        pages.append(PageData(
            page_number=page_num + 1,
            width=rect.width,
            height=rect.height,
            text=page_text,
            blocks=blocks,
        ))

    headings = extract_headings(pages)
    markdown = blocks_to_markdown(pages)

    language = None
    sample = markdown[:2000]
    cjk_count = sum(1 for c in sample if "\u4e00" <= c <= "\u9fff")
    if cjk_count > len(sample) * 0.1:
        language = "zh"
    elif sample:
        language = "en"

    metadata = DocumentMetadata(
        title=meta.get("title") or None,
        author=meta.get("author") or None,
        subject=meta.get("subject") or None,
        creator=meta.get("creator") or None,
        page_count=len(doc),
        language=language,
    )

    doc.close()
    return ParseResult(
        markdown=markdown,
        pages=pages,
        headings=headings,
        metadata=metadata,
    )
