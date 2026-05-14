import logging
import sys
import time
import uuid

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

from app.routers import parse, convert, ocr

logging.basicConfig(
    level=logging.INFO,
    format='{"timestamp":"%(asctime)s","level":"%(levelname)s","logger":"%(name)s","message":"%(message)s"}',
    datefmt="%Y-%m-%dT%H:%M:%S",
    stream=sys.stdout,
)
logger = logging.getLogger("trustrag.doc-processor")

app = FastAPI(
    title="TrustRAG Document Processor",
    version="0.1.0",
    description="Python sidecar service for document parsing, conversion, and OCR",
)


@app.middleware("http")
async def logging_middleware(request: Request, call_next):
    request_id = str(uuid.uuid4())[:8]
    start = time.time()
    logger.info(
        f"request_start request_id={request_id} method={request.method} path={request.url.path}"
    )
    try:
        response = await call_next(request)
        duration_ms = int((time.time() - start) * 1000)
        logger.info(
            f"request_end request_id={request_id} status={response.status_code} duration_ms={duration_ms}"
        )
        response.headers["X-Request-ID"] = request_id
        return response
    except Exception as e:
        duration_ms = int((time.time() - start) * 1000)
        logger.error(
            f"request_error request_id={request_id} error={type(e).__name__}: {e} duration_ms={duration_ms}"
        )
        return JSONResponse(
            status_code=500,
            content={"error": "Internal server error", "request_id": request_id},
        )


app.include_router(parse.router, prefix="/api/parse", tags=["parse"])
app.include_router(convert.router, prefix="/api/convert", tags=["convert"])
app.include_router(ocr.router, prefix="/api/ocr", tags=["ocr"])


@app.get("/health")
async def health():
    return {"status": "ok"}
