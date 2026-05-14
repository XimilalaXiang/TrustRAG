from fastapi import APIRouter, UploadFile, File

router = APIRouter()


@router.post("/pdf")
async def parse_pdf(file: UploadFile = File(...)):
    """Parse a PDF file and return structured Markdown with position mapping."""
    pass


@router.post("/docx")
async def parse_docx(file: UploadFile = File(...)):
    """Parse a DOCX file and return structured Markdown."""
    pass


@router.post("/txt")
async def parse_txt(file: UploadFile = File(...)):
    """Parse a plain text file."""
    pass
