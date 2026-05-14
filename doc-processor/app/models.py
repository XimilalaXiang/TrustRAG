from pydantic import BaseModel
from typing import Optional


class TextBlock(BaseModel):
    type: str  # "text", "heading", "table", "image"
    content: str
    bbox: list[float]  # [x0, y0, x1, y1]
    heading_level: Optional[int] = None
    font_size: Optional[float] = None
    is_bold: Optional[bool] = None


class PageData(BaseModel):
    page_number: int
    width: float
    height: float
    text: str
    blocks: list[TextBlock]


class HeadingInfo(BaseModel):
    text: str
    level: int
    page: int
    block_index: int = 0


class DocumentMetadata(BaseModel):
    title: Optional[str] = None
    author: Optional[str] = None
    subject: Optional[str] = None
    creator: Optional[str] = None
    page_count: int = 0
    language: Optional[str] = None


class ParseResult(BaseModel):
    markdown: str
    pages: list[PageData]
    headings: list[HeadingInfo]
    metadata: DocumentMetadata
