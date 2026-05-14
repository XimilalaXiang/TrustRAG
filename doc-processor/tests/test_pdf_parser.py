"""Tests for PDF parser using a dynamically generated PDF."""
import fitz  # PyMuPDF

from app.processors.pdf import parse_pdf, detect_heading_level


def make_test_pdf() -> bytes:
    """Create a minimal PDF with headings and body text for testing."""
    doc = fitz.open()

    page = doc.new_page(width=595, height=842)
    page.insert_text((72, 72), "Test Document Title", fontsize=20, fontname="helv")
    page.insert_text((72, 120), "Chapter 1 Introduction", fontsize=16, fontname="helv")
    page.insert_text((72, 160), "This is the first paragraph of the introduction. "
                     "It contains some text to test the parser.", fontsize=11, fontname="helv")
    page.insert_text((72, 200), "1.1 Background", fontsize=14, fontname="helv")
    page.insert_text((72, 240), "Background text goes here with details about the topic.",
                     fontsize=11, fontname="helv")

    page2 = doc.new_page(width=595, height=842)
    page2.insert_text((72, 72), "Chapter 2 Methods", fontsize=16, fontname="helv")
    page2.insert_text((72, 120), "Methods description paragraph.",
                      fontsize=11, fontname="helv")

    pdf_bytes = doc.tobytes()
    doc.close()
    return pdf_bytes


def test_parse_pdf_basic():
    pdf_bytes = make_test_pdf()
    result = parse_pdf(pdf_bytes)

    assert result.metadata.page_count == 2
    assert len(result.pages) == 2
    assert result.pages[0].page_number == 1
    assert result.pages[1].page_number == 2
    assert len(result.markdown) > 0


def test_parse_pdf_pages_have_blocks():
    pdf_bytes = make_test_pdf()
    result = parse_pdf(pdf_bytes)

    page1 = result.pages[0]
    assert len(page1.blocks) > 0

    for block in page1.blocks:
        assert block.content
        assert len(block.bbox) == 4


def test_parse_pdf_headings_detected():
    pdf_bytes = make_test_pdf()
    result = parse_pdf(pdf_bytes)

    assert len(result.headings) > 0
    heading_texts = [h.text for h in result.headings]
    assert any("Chapter" in t or "Introduction" in t for t in heading_texts)


def test_parse_pdf_markdown_contains_text():
    pdf_bytes = make_test_pdf()
    result = parse_pdf(pdf_bytes)

    assert "Introduction" in result.markdown or "introduction" in result.markdown.lower()
    assert "Methods" in result.markdown or "methods" in result.markdown.lower()


def test_parse_pdf_metadata():
    pdf_bytes = make_test_pdf()
    result = parse_pdf(pdf_bytes)

    assert result.metadata.page_count == 2
    assert result.metadata.language == "en"


def test_detect_heading_level_large_font():
    level = detect_heading_level("Big Title", 20.0, True, [11.0, 11.0, 11.0])
    assert level == 1


def test_detect_heading_level_normal_text():
    level = detect_heading_level(
        "This is normal body text that should not be a heading at all.",
        11.0, False, [11.0, 11.0, 11.0]
    )
    assert level is None


def test_detect_heading_level_chapter_pattern():
    level = detect_heading_level("第一章 概述", 13.0, True, [11.0, 11.0])
    assert level is not None
    assert level <= 3
