# ADR-0002: Split-Plane Binary File Streaming with Sliding-Window Flow Control

## Context
Transferring files ranging from small photos to large 2GB+ video files over RFCOMM Bluetooth and LAN WebSockets risks memory exhaustion (OOM), bridge latency, and buffer overrun on slow serial connections.

## Decision
We split file transfers into two distinct architectural planes:
1. **Control Plane (`DataType::Packet` [0x06])**: Handles metadata negotiation (`file_name`, `file_size`, `mime_type`, `transfer_id`) and user acceptance dialogs over JSON packets.
2. **Data Plane (`DataType::FileChunk` [0x07] & `ChunkAck` [0x09])**: Operates directly on native socket threads, streaming binary chunks directly to disk (`~/Downloads/.tools_staging_{transfer_id}.part`), bypassing frontend bridge memory.

We enforce **sliding-window flow control** with window size $W=8$ (32 KB in-flight) for RFCOMM and $W=32$ (2 MB in-flight) for WebSockets. The receiver verifies CRC-32 checksums per chunk and on EOF atomically promotes the staging file to `~/Downloads/{file_name}`. Progress notifications are throttled to at most once every 100ms.

## Consequences
- Constant $O(1)$ memory footprint regardless of file size (<1.2 MB).
- Eliminates Bluetooth RFCOMM buffer overflow and packet drops.
- Partial transfers cannot corrupt existing files due to atomic `.part` promotion.
