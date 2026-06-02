# AGENTS.md

## Mandatory MCP tools

- **ALWAYS use `context7`** (MCP server) to get up-to-date code examples and documentation for the libraries being used. Never rely only on general knowledge.

## Documentation

- Written in English, in Markdown format.
- Entry point: `README.md`.
- AI-generated files must be placed in `documentation/ai/`.
- Technical documentation must be placed in `documentation/technical/`.
- Mermaid diagrams:
  - Placed in `.md` files, not `.mmd` files.
  - Arrow conventions: `-->` for 1-N, `<-->` for N-N, `-.->` for nullable.
  - Clear, concise, and well commented.