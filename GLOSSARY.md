# Tools Desktop Domain Glossary

Tools Desktop is the desktop companion application built on Tauri 2 and Rust, acting as a local hub for multi-device decentralized peer-to-peer productivity tooling.

## Language

**ActiveNoteSession**:
An in-memory session holding an open note's `automerge::AutoCommit` document for real-time keystroke processing, debouncing writes to SQLite by 500ms.
_Avoid_: Open note buffer, cached document

**AutomergeSync**:
The binary framing format (`DataType::AutomergeSync` = `0x08`) carrying a 2-byte length-prefixed note ID and raw Automerge sync state bytes between peers.
_Avoid_: Note packet, CRDT update

**BlockRecord**:
The normalized relational representation of a BlockNote block within the Automerge document map (`id`, `type`, `props`, `content`, `children`).
_Avoid_: ProseMirror node, block JSON

**BondedCompanion**:
A recognized peer device that has successfully completed the cryptographic token handshake and is trusted for automatic file transfer acceptance.
_Avoid_: Paired host, trusted client

**ChunkAck**:
The 28-byte sliding-window flow control feedback frame (`DataType::ChunkAck` = `0x09`) acknowledging received chunks and providing window credits.
_Avoid_: File receipt, transfer ACK

**FileChunk**:
The binary frame layout (`DataType::FileChunk` = `0x07`) containing a 41-byte binary header followed by raw payload bytes streamed directly to disk.
_Avoid_: File packet, binary slice

**EchoCancellation**:
The mathematical suppression of redundant inbound deltas or clipboard loops using Automerge `SyncState` or dual circular SHA-256 hash ring buffers.
_Avoid_: Loop filter, dedup

**SyncState**:
The state machine tracked per peer connection recording the heads and Bloom filters exchanged, reset cleanly upon reconnection.
_Avoid_: Peer session, sync cursor
