---
name: graphify
description: Transform repository codebases, schemas, and markdown docs into queryable structural knowledge graphs and AST relationship maps without vector databases.
---

# Graphify Skill

Graphify parses an entire repository's source code, markdown documentation, configurations, and schemas into a deterministic structural knowledge graph using AST parsing (Tree-sitter) and semantic linking.

## Capabilities

1. **Deterministic AST Code Graphing**:
   - Maps functions, structs, classes, modules, and their invocation call-graphs.
   - Traces callers and callees across repository files without vector search guesswork.

2. **Community Detection & Clustering**:
   - Groups related components into architectural communities (e.g., Core Storage, Input Engine, Native Activity Bridge).
   - Identifies heavily connected "god nodes" and circular dependencies.

3. **Knowledge Graph Generation**:
   - Generates Mermaid diagram graphs, DOT files, and JSON network maps.
   - Exports Obsidian-compatible wikilinks `[[concept]]` and notes for graph view.

## Workflows

### Generating a Structural Code Graph
When analyzing a repository architecture:
1. Identify all package/crate root modules.
2. Parse exported types, traits, structs, and public functions.
3. Trace data dependencies and module dependencies.
4. Output structured Mermaid graphs and dependency matrices.
