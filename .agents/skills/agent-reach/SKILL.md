---
name: agent-reach
description: Multi-source internet intelligence gathering and external information retrieval across web pages, docs, repositories, community platforms, and API endpoints.
---

# Agent Reach Skill

Use this skill to research external references, official documentation, open-source repositories, and technical specifications across the internet.

## Core Capabilities

1. **Web Document Parsing & Scraping**:
   - Extract raw markdown and text from technical websites, SDK documentations, and GitHub issues.
   - Utilize Jina Reader (`r.jina.ai/<url>`) or native HTTP fetchers to retrieve clean, advertising-free markdown.

2. **Repository & Upstream Intelligence**:
   - Query upstream repositories, release changelogs, commit diffs, and developer discussions.
   - Trace upstream breaking changes and API deprecations.

3. **Multi-Source Verification**:
   - Cross-verify technical details across official documentation, source code, and known issue trackers before implementing complex platform layers.
