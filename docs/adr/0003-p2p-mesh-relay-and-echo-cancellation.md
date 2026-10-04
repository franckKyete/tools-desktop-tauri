# ADR-0003: P2P Mesh Sibling Relay with Automerge SyncState Echo Cancellation

## Context
In a decentralized multi-device mesh with 3+ companions (e.g., 2 PCs, 1 Phone) connected peer-to-peer over Bluetooth RFCOMM and WebSockets, an edit originating on Device A must propagate to Device C via Device B without bouncing back to Device A or causing infinite relay loops.

## Decision
We implemented a non-blocking Tokio MPSC sync channel (`InboundSyncTask`) to decouple socket readers from document locks. A dedicated `AutomergeSyncWorker` applies incoming changes to the local document and fans out outbound sync deltas to all connected sibling bridges (`sibling_peer_id != source_peer_id`).

Echo loops back to the sender are mathematically suppressed: when Device B receives changes from Device A, Device A's heads are already recorded in `peer_sync_states[(note_id, device_a)]`, causing `doc.generate_sync_message(device_a_state)` to return `None` naturally. 

We maintain ephemeral in-memory `SyncState` per peer connection session: on disconnect, entries are purged; on reconnect, a fresh `SyncState::new()` starts clean, resolving heads via bloom filters without risk of stale state divergence.

## Consequences
- Zero-overhead echo cancellation without requiring custom vector clocks or message ID deduplication tables.
- Resilient multi-hop routing across heterogeneous transports (RFCOMM + WebSocket).
- Inactive notes are synced lazily or throttled, preventing initial sync storms on connect.
