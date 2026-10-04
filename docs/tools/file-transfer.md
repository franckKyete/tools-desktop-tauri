# Desktop File Transfer Tool

The **Desktop File Transfer Tool** provides high-throughput, memory-safe binary file exchanges with mobile companions over Bluetooth RFCOMM and LAN WebSockets without relying on third-party cloud services.

---

## Architecture

```mermaid
sequenceDiagram
    autonumber
    participant UI as Desktop React UI
    participant Bridge as Bridge Transport
    participant Daemon as File Transfer Daemon
    participant Disk as Local Disk (~/Downloads)
    participant Peer as Mobile Companion

    Note over UI,Peer: Phase 1: Control Plane (DataType::Packet 0x06)
    UI->>Bridge: Send FileRequest (transfer_id, file_name, file_size, mime_type)
    Bridge->>Peer: DataType::Packet (FileRequestPayload)
    Peer-->>Bridge: DataType::Packet (FileResponsePayload: accepted=true)
    Bridge-->>UI: Transfer Approved

    Note over UI,Peer: Phase 2: Data Plane (DataType::FileChunk 0x07 & ChunkAck 0x09)
    loop Sliding-Window Streaming (W=8 RFCOMM / W=32 WS)
        Bridge->>Peer: DataType::FileChunk [0x07] (41B Header + Binary Chunk)
        Peer-->>Bridge: DataType::ChunkAck [0x09] (28B Window Credit Feedback)
        opt Throttled (Max 100ms)
            Bridge->>UI: emit("file_transfer_progress", {transferId, bytes, percent, speed})
        end
    end

    Note over UI,Disk: Phase 3: Finalization & Promotion
    Bridge->>Disk: Verify CRC-32 & File Size
    Bridge->>Disk: Atomically rename .part -> ~/Downloads/{file_name}
    Bridge->>UI: emit("file_transfer_complete", {transferId, filePath})
```

---

## 1. Split-Plane Architecture

1. **Control Plane (`DataType::Packet` [0x06])**:
   - Manages request/response negotiation (`FileRequestPayload` and `FileResponsePayload`), authentication verification, and user dialogs.
   - Bonded companions auto-accept transfers; untrusted peers prompt confirmation.
2. **Data Plane (`DataType::FileChunk` [0x07] & `ChunkAck` [0x09])**:
   - Operates directly on native socket threads (`socket.rs`).
   - Bypasses Tauri frontend IPC serialization, maintaining constant $O(1)$ memory usage (< 1.2 MB).

---

## 2. Sliding-Window Flow Control

To saturate available network bandwidth without overflowing serial Bluetooth buffers:
- **Bluetooth RFCOMM**: Window size $W = 8$ unacknowledged chunks (chunk size: 4 KB to 8 KB).
- **WebSocket (LAN)**: Window size $W = 32$ unacknowledged chunks (chunk size: 64 KB to 128 KB).
- The receiver replenishes window credits via `DataType::ChunkAck` (`0x09`) whenever in-flight credits fall to $\le W/2$.

---

## 3. Staging & Atomic File Promotion

- **Staging Location**: Incoming chunks are appended to a temporary file on the same filesystem:
  `~/Downloads/.tools_staging_{transfer_id}.part`.
- **Integrity Verification**: IEEE CRC-32 checksums are verified per chunk and across the total file on EOF (`CHUNK_FLAG_EOF`).
- **Atomic Promotion**: Upon successful EOF verification, the daemon atomically renames `.part` to `~/Downloads/{file_name}`. Failed or cancelled transfers remove the staging file immediately.
