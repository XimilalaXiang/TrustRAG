from fastapi import APIRouter, UploadFile, File

router = APIRouter()


@router.post("/extract")
async def ocr_extract(file: UploadFile = File(...)):
    """Extract text from scanned images/PDFs using Tesseract OCR."""
    pass
