from fastapi import APIRouter

router = APIRouter()


@router.post("/to-markdown")
async def convert_to_markdown():
    """Convert various formats to Markdown using Pandoc."""
    pass
