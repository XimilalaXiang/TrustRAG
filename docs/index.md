---
layout: home
hero:
  name: TrustRAG
  text: Trustworthy RAG Knowledge Workbench
  tagline: Upload documents. Ask questions. Trace every answer back to its source — with verifiable citations, review workflows, and a self-contained desktop app.
  actions:
    - theme: brand
      text: Download v0.1.2
      link: https://github.com/XimilalaXiang/TrustRAG/releases/latest
    - theme: alt
      text: Get Started →
      link: /guide/getting-started
    - theme: alt
      text: API Reference
      link: /api/overview
features:
  - icon: 📎
    title: Citation Tracking
    details: Every AI response includes traceable citations linked to document, chunk, page, and heading. Never trust an answer blindly — verify it.
  - icon: ✅
    title: Citation Review
    details: Approve, reject, or flag citations for accuracy. Full review history for accountability and continuous improvement.
  - icon: 🖥️
    title: Multi-Platform Desktop
    details: Windows (.exe installer + portable), macOS, Linux, Android, iOS, and Web — all from a single Flutter codebase.
  - icon: 📦
    title: Self-Contained Desktop
    details: Embedded SQLite + Rust backend inside the app. No external database, no server setup. Download, install, and start asking questions.
  - icon: 🤖
    title: RAG Pipeline
    details: Document-grounded retrieval-augmented generation with configurable LLM and embedding providers. Hybrid search with vector + full-text retrieval.
  - icon: 🧠
    title: Knowledge Graph
    details: Entity and relation extraction for cross-document exploration. Build connections between concepts across your entire knowledge base.
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #10b981 30%, #3b82f6);
  --vp-home-hero-image-background-image: linear-gradient(-45deg, #10b98140 50%, #3b82f640 50%);
  --vp-home-hero-image-filter: blur(44px);
}
</style>
