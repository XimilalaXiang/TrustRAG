"""Position mapper: maps text chunks back to original document locations.

Given parse output (pages with blocks), builds an index that allows mapping
any text substring back to its page number, bounding box, paragraph index,
character offsets, and heading path.
"""
from dataclasses import dataclass, field

from app.models import HeadingInfo, PageData


@dataclass
class BlockPosition:
    page_number: int
    block_index: int
    bbox: list[float]
    heading_path: str
    char_start: int  # global character offset in the full text
    char_end: int


@dataclass
class ChunkPosition:
    page_start: int
    page_end: int
    paragraph_index: int
    char_start: int
    char_end: int
    heading_path: str
    bbox_start: list[float]
    bbox_end: list[float]


@dataclass
class PositionIndex:
    """Pre-built index for fast chunk-to-position lookups."""
    blocks: list[BlockPosition] = field(default_factory=list)
    full_text: str = ""


def build_heading_path(headings_so_far: list[tuple[int, str]]) -> str:
    """Build a hierarchical heading path like '第1章 > 1.1 背景 > 1.1.2 详情'."""
    if not headings_so_far:
        return ""
    return " > ".join(text for _, text in headings_so_far)


def build_position_index(pages: list[PageData], headings: list[HeadingInfo]) -> PositionIndex:
    """Build a position index from parsed page data.

    Walks through all blocks in page order, tracking cumulative character offsets
    and the current heading hierarchy.
    """
    blocks: list[BlockPosition] = []
    full_text_parts: list[str] = []
    char_offset = 0
    heading_stack: list[tuple[int, str]] = []  # [(level, text), ...]

    for page in pages:
        for block_idx, block in enumerate(page.blocks):
            content = block.content

            if block.heading_level is not None:
                level = block.heading_level
                heading_stack = [
                    (lvl, txt) for lvl, txt in heading_stack if lvl < level
                ]
                heading_stack.append((level, content))

            heading_path = build_heading_path(heading_stack)
            char_start = char_offset
            char_end = char_offset + len(content)

            blocks.append(BlockPosition(
                page_number=page.page_number,
                block_index=block_idx,
                bbox=block.bbox,
                heading_path=heading_path,
                char_start=char_start,
                char_end=char_end,
            ))

            full_text_parts.append(content)
            char_offset = char_end + 1  # +1 for separator

    full_text = "\n".join(full_text_parts)

    return PositionIndex(blocks=blocks, full_text=full_text)


def locate_chunk(index: PositionIndex, chunk_text: str) -> ChunkPosition | None:
    """Find the position of a text chunk in the document.

    Uses string search on the full text, then maps the character range
    back to block positions.
    """
    pos = index.full_text.find(chunk_text)
    if pos == -1:
        normalized_chunk = " ".join(chunk_text.split())
        normalized_full = " ".join(index.full_text.split())
        pos = normalized_full.find(normalized_chunk)
        if pos == -1:
            return None

    chunk_start = pos
    chunk_end = pos + len(chunk_text)

    start_block = None
    end_block = None
    para_index = 0

    for i, block in enumerate(index.blocks):
        if block.char_start <= chunk_start < block.char_end:
            start_block = block
            para_index = i
        if block.char_start < chunk_end <= block.char_end:
            end_block = block
            break

    if start_block is None:
        for i, block in enumerate(index.blocks):
            if block.char_end >= chunk_start:
                start_block = block
                para_index = i
                break

    if end_block is None:
        for block in reversed(index.blocks):
            if block.char_start <= chunk_end:
                end_block = block
                break

    if start_block is None or end_block is None:
        return None

    return ChunkPosition(
        page_start=start_block.page_number,
        page_end=end_block.page_number,
        paragraph_index=para_index,
        char_start=chunk_start,
        char_end=chunk_end,
        heading_path=start_block.heading_path,
        bbox_start=start_block.bbox,
        bbox_end=end_block.bbox,
    )


def map_chunks(
    pages: list[PageData],
    headings: list[HeadingInfo],
    chunks: list[str],
) -> list[ChunkPosition | None]:
    """Map a list of text chunks to their positions in the document."""
    index = build_position_index(pages, headings)
    return [locate_chunk(index, chunk) for chunk in chunks]
