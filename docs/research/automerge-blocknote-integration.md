# Research Report: Automerge-rs Integration with BlockNote Document Model

**Issue Reference**: [#2 Research: Automerge-rs integration with BlockNote document model](https://github.com/franckKyete/tools-desktop-tauri/issues/2)  
**Parent Epic**: [#1 Wayfinder Map: Complete Phase 1 Core Trio](https://github.com/franckKyete/tools-desktop-tauri/issues/1)  
**Downstream Dependency**: [#4 Prototype: Peer-to-peer Automerge binary update relaying in BridgeManager](https://github.com/franckKyete/tools-desktop-tauri/issues/4)  

---

## Executive Summary

This report resolves the architectural question posed in **Issue #2**:
> *How should Automerge-rs (Rust backend in `tools-desktop-tauri`) and BlockNote (React frontend in `tools-desktop-tauri`) represent, serialize, and synchronize rich-text block trees, and what is the optimal sync wire format between frontend IPC and backend storage?*

### Key Conclusions & Architecture Overview

1. **Document Model Schema**:
   - Represent documents in Automerge using a **Normalized Relational Block CRDT Schema** (`blocks: Map<BlockId, BlockData>`, `root_order: List<BlockId>`).
   - Text within each block is an independent Automerge `ObjType::Text` annotated with Peritext **marks** (`bold`, `italic`, `link`).
   - Indented blocks are represented with `children_order: List<BlockId>`.
2. **Optimal Wire Format**:
   - **Tauri IPC Wire Format**: Raw binary `Uint8Array` / `Vec<u8>` encoding of `automerge::sync::Message`. Zero JSON diffing, zero base64 expansion.
   - **Bridge Transport Wire Format (Desktop <-> Mobile)**: Dedicated binary framing `DataType::AutomergeSync = 0x07` (`[DataType (1B)][NoteIdLen (2B)][NoteId (N bytes)][AutomergeSyncMessage (M bytes)]`). Eliminates 33% base64 overhead over Bluetooth RFCOMM.
3. **Multi-Peer Relaying (Issue #4)**:
   - `BridgeManager` maintains `peer_states: HashMap<(NoteId, PeerId), automerge::sync::State>`.
   - Incoming sync frames are applied to `AutoCommit`, generating outbound sync messages for other connected peers automatically with zero echo loop.
4. **Storage Engine (SQLite)**:
   - **Debounced Atomic Snapshot persistence** (`doc.save() -> Vec<u8>`) in SQLite, coupled with an in-memory `AutoCommit` cache in `NoteManager`.
   - Debounced by 500ms to disk while dispatched in real time (<5ms) over P2P bridges.

---

## Automerge Data Model

```json
{
  "id": "note-550e8400-e29b-41d4-a716-446655440000",
  "title": "Project Architecture",
  "root_order": ["block-001", "block-003"],
  "blocks": {
    "block-001": {
      "id": "block-001",
      "parent_id": null,
      "type": "paragraph",
      "props": {
        "textColor": "default",
        "backgroundColor": "default",
        "textAlignment": "left"
      },
      "content": "Hello world with bold text",
      "children_order": ["block-002"]
    },
    "block-002": {
      "id": "block-002",
      "parent_id": "block-001",
      "type": "bulletListItem",
      "props": {},
      "content": "Indented child item",
      "children_order": []
    }
  }
}
```

---

## Wire Format Specification: DataType 0x07 (AutomergeSync)

```
Offset  Size  Field              Type / Description
-----------------------------------------------------------
0x00    1     DataType           0x07 (AutomergeSync)
0x01    2     note_id_len        u16 (Big-Endian)
0x03    N     note_id            UTF-8 String (note_id_len bytes)
0x03+N  M     sync_message       Raw Automerge sync message (Message::encode())
```

---

## Action Items for Downstream Issues

1. **For Issue #4 (Prototype: Peer-to-peer Automerge binary update relaying in BridgeManager)**:
   - Implement `DataType::AutomergeSync = 0x07` in `bin_data.rs`.
   - Add `peer_states: HashMap<(String, String), automerge::sync::State>` to `BridgeManager`.
   - Implement relay loop broadcasting sync messages to all connected peers except the sender.
2. **For Issue #5 (Wire Types Task)**:
   - Include `Payload::AutomergeSync(Vec<u8>)` and `DataType::AutomergeSync = 0x07`.
