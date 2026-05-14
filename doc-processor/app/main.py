from fastapi import FastAPI

from app.routers import parse, convert, ocr

app = FastAPI(
    title="TrustRAG Document Processor",
    version="0.1.0",
    description="Python sidecar service for document parsing, conversion, and OCR",
)

app.include_router(parse.router, prefix="/api/parse", tags=["parse"])
app.include_router(convert.router, prefix="/api/convert", tags=["convert"])
app.include_router(ocr.router, prefix="/api/ocr", tags=["ocr"])


@app.get("/health")
async def health():
    return {"status": "ok"}
