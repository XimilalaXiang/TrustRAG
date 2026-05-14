"""Tests for position mapper module."""
from app.models import HeadingInfo, PageData, TextBlock
from app.processors.position_mapper import (
    build_heading_path,
    build_position_index,
    locate_chunk,
    map_chunks,
)


def make_test_pages() -> tuple[list[PageData], list[HeadingInfo]]:
    """Create test page data with headings and text blocks."""
    pages = [
        PageData(
            page_number=1,
            width=595,
            height=842,
            text="Chapter 1 Introduction\nThis is the introduction.",
            blocks=[
                TextBlock(
                    type="heading", content="Chapter 1 Introduction",
                    bbox=[72, 72, 500, 100], heading_level=1,
                ),
                TextBlock(
                    type="text", content="This is the introduction.",
                    bbox=[72, 120, 500, 150],
                ),
            ],
        ),
        PageData(
            page_number=2,
            width=595,
            height=842,
            text="1.1 Background\nBackground details here.\nChapter 2 Methods\nMethods text.",
            blocks=[
                TextBlock(
                    type="heading", content="1.1 Background",
                    bbox=[72, 72, 300, 100], heading_level=2,
                ),
                TextBlock(
                    type="text", content="Background details here.",
                    bbox=[72, 120, 500, 150],
                ),
                TextBlock(
                    type="heading", content="Chapter 2 Methods",
                    bbox=[72, 200, 500, 230], heading_level=1,
                ),
                TextBlock(
                    type="text", content="Methods text.",
                    bbox=[72, 250, 300, 270],
                ),
            ],
        ),
    ]
    headings = [
        HeadingInfo(text="Chapter 1 Introduction", level=1, page=1, block_index=0),
        HeadingInfo(text="1.1 Background", level=2, page=2, block_index=0),
        HeadingInfo(text="Chapter 2 Methods", level=1, page=2, block_index=2),
    ]
    return pages, headings


def test_build_heading_path():
    path = build_heading_path([(1, "Chapter 1"), (2, "1.1 Background")])
    assert path == "Chapter 1 > 1.1 Background"


def test_build_heading_path_empty():
    path = build_heading_path([])
    assert path == ""


def test_build_position_index():
    pages, headings = make_test_pages()
    index = build_position_index(pages, headings)

    assert len(index.blocks) == 6
    assert index.blocks[0].heading_path == "Chapter 1 Introduction"
    assert index.blocks[0].page_number == 1

    assert "introduction" in index.full_text.lower()
    assert "background" in index.full_text.lower()


def test_build_position_index_heading_hierarchy():
    pages, headings = make_test_pages()
    index = build_position_index(pages, headings)

    bg_block = index.blocks[3]
    assert "1.1 Background" in bg_block.heading_path
    assert "Chapter 1" in bg_block.heading_path

    methods_block = index.blocks[5]
    assert "Chapter 2" in methods_block.heading_path
    assert "1.1 Background" not in methods_block.heading_path


def test_locate_chunk_exact():
    pages, headings = make_test_pages()
    index = build_position_index(pages, headings)

    pos = locate_chunk(index, "This is the introduction.")
    assert pos is not None
    assert pos.page_start == 1
    assert "Chapter 1" in pos.heading_path


def test_locate_chunk_cross_page():
    pages, headings = make_test_pages()
    index = build_position_index(pages, headings)

    pos = locate_chunk(index, "Background details here.")
    assert pos is not None
    assert pos.page_start == 2


def test_locate_chunk_not_found():
    pages, headings = make_test_pages()
    index = build_position_index(pages, headings)

    pos = locate_chunk(index, "This text does not exist anywhere.")
    assert pos is None


def test_map_chunks():
    pages, headings = make_test_pages()
    chunks = ["This is the introduction.", "Methods text.", "nonexistent"]
    results = map_chunks(pages, headings, chunks)

    assert len(results) == 3
    assert results[0] is not None
    assert results[0].page_start == 1
    assert results[1] is not None
    assert results[1].page_start == 2
    assert results[2] is None
