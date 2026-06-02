---
name: documentation
description: In-code documentation, folder READMEs, and code comments. Use when writing README.md files, JSDoc comments, or explaining code organization.
---

# Documentation

Follow [writing-voice](../writing-voice/SKILL.md) for tone.

Documentation explains **why**, not **what**. Users can read code to see what it does. They need you to explain the reasoning.

When documentation is written in French, write proper French:

- use accents and diacritics correctly
- avoid plain-ASCII approximations like `etre`, `deploiement`, or `reponse`
- keep spelling, typography, and terminology polished enough for end-user reading

## Folder READMEs

Primary job: explain **why** this folder exists and the mental model.

### Can Include

- ASCII art diagrams for complex relationships
- Overview of key exports or entry points
- Brief file descriptions IF they add context beyond the filename
- Relationships to other folders

### Avoid

- Exhaustive file listings that just duplicate `ls`
- Descriptions that repeat the filename ("auth.ts - authentication")
- Implementation details better expressed in code

### Good

````markdown
# Converters

Transform field schemas into format-specific representations.

```
┌─────────────┐     ┌──────────────┐
│ Field Schema│────▶│  to-arktype  │────▶ Runtime validation
└─────────────┘     ├──────────────┤
                    │  to-drizzle  │────▶ SQLite columns
                    └──────────────┘
```

Field schemas are pure JSON Schema objects with `x-component` hints. Each converter takes the same input and produces output for a specific consumer.
````

### Bad

```markdown
# Converters

- `to-arktype.ts` - Converts to ArkType
- `to-drizzle.ts` - Converts to Drizzle
- `index.ts` - Exports
```

The bad example just lists files without explaining the pattern or when to add new converters.

## Code Comments

Comments explain **why**, not **what**.

### Good

```typescript
// Y.Doc clientIDs are random 32-bit integers, so we can't rely on ordering.
// Use timestamps from the entries themselves for deterministic sorting.
const sorted = entries.sort((a, b) => a.timestamp - b.timestamp);
```

### Bad

```typescript
// Sort the entries
const sorted = entries.sort((a, b) => a.timestamp - b.timestamp);
```

### Rules

- If the code is clear, don't comment it
- Comment the "why" when it's not obvious
- Comment workarounds with links to issues/docs
- Delete commented-out code; that's what git is for
