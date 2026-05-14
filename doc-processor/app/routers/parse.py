from fastapi import APIRouter, UploadFile, File, HTTPException

from app.models import ParseResult
from app.processors.pdf import parse_pdf

router = APIRouter()

MAX_FILE_SIZE = 100 * 1024 * 1024  # 100 MB


@router.post("/pdf", response_model=ParseResult)
async def parse_pdf_endpoint(file: UploadFile = File(...)):
    """Parse a PDF file and return structured Markdown with position mapping."""
    if not file.filename or not file.filename.lower().endswith(".pdf"):
        raise HTTPException(status_code=400, detail="File must be a PDF")

    content = await file.read()
    if len(content) > MAX_FILE_SIZE:
        raise HTTPException(status_code=400, detail="File too large (max 100MB)")

    if len(content) < 4 or content[:4] != b"%PDF":
        raise HTTPException(status_code=400, detail="Invalid PDF file")

    try:
        result = parse_pdf(content)
    except Exception as e:
        raise HTTPException(status_code=422, detail=f"Failed to parse PDF: {e}")

    return result


@router.post("/docx", response_model=ParseResult)
async def parse_docx_endpoint(file: UploadFile = File(...)):
    """Parse a DOCX file and return structured Markdown."""
    raise HTTPException(status_code=501, detail="DOCX parsing not yet implemented")


@router.post("/txt", response_model=ParseResult)
async def parse_txt_endpoint(file: UploadFile = File(...)):
    """Parse a plain text file."""
    if not file.filename:
        raise HTTPException(status_code=400, detail="File must have a name")

    content = await file.read()
    text = content.decode("utf-8", errors="replace")

    from app.models import DocumentMetadata, PageData, TextBlock

    block = TextBlock(type="text", content=text, bbox=[0, 0, 0, 0])
    page = PageData(page_number=1, width=0, height=0, text=text, blocks=[block])

    return ParseResult(
        markdown=text,
        pages=[page],
        headings=[],
        metadata=DocumentMetadata(
            title=file.filename,
            page_count=1,
            language=None,
        ),
    )
