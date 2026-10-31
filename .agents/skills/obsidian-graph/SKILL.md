---
name: obsidian-graph
description: Generate and maintain an Obsidian-compatible knowledge vault with bidirectionally linked markdown notes, tags, frontmatter, and graph view topologies.
---

# Obsidian Graph Skill

Use this skill when generating documentation, architecture decision records (ADRs), or codebase knowledge bases formatted for Obsidian vaults and interactive graph visualization.

## Guidelines

1. **Wikilinks**:
   - Use `[[Note Name]]` or `[[Target Note|Custom Label]]` for bidirectional cross-references.
   - Link architectural concepts to the underlying source modules (e.g., `[[Touch Engine]] -> [[launcher-core]]`).

2. **Frontmatter Metadata**:
   - Always include standard YAML frontmatter:
     ```yaml
     ---
     title: Note Title
     tags: [architecture, rust, android, mobile]
     created: YYYY-MM-DD
     status: active | draft | deprecated
     aliases: [Alternative Name]
     ---
     ```

3. **Graph Topology Organization**:
   - Create hub / Index notes (`MOC - Map of Content`) connecting related nodes.
   - Embed Mermaid graphs for inline rendering alongside Obsidian's native 2D/3D force-directed graph view.
