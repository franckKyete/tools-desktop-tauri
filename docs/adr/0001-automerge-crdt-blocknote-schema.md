# ADR-0001: Automerge CRDT with Relational Block Schema for Rich Text Notes

## Context
Tools Notes requires real-time, conflict-free multi-device synchronization across peer-to-peer links (Bluetooth RFCOMM and WebSockets). The rich-text editor is BlockNote (ProseMirror-based). We needed to decide how rich text is modeled in a CRDT, where the CRDT engine executes, and how documents persist locally.

## Decision
We adopted **Automerge** as the authoritative CRDT engine. In `tools-desktop-tauri`, Automerge runs natively in the Rust backend (`automerge-rs`), exposing materialized BlockNote `Block[]` JSON trees across Tauri IPC commands and events (`open_note`, `update_note_blocks`, `note_blocks_updated`). 

Documents are modeled using a **normalized relational block schema** (`blocks: Map<BlockId, BlockData>` and `block_order: List<BlockId>`) rather than raw ProseMirror steps, preventing block ID collisions and enabling fine-grained character-level text merging inside block contents.

Document lifecycle uses an **in-memory active session** (`ActiveNoteSession`) for open notes with a 500ms trailing debounce before flushing `automerge::AutoCommit` snapshots to SQLite BLOB storage. Inactive notes are loaded, merged, saved, and dropped on-demand during P2P sync.

## Consequences
- Zero WebAssembly runtime required in the desktop React frontend.
- Instant sub-millisecond typing latency in memory with reduced SQLite disk I/O.
- Clean separation between presentation (`Block[]` JSON) and replication (binary Automerge sync messages).
