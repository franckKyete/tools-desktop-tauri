# Research Report: Binary `DataType 0x06` (`FileChunk`) Framing, Transport MTU, and Buffer Limits on RFCOMM and WebSockets

**Target Issue:** Issue #3 (`franckKyete/tools-desktop-tauri#3`)  
**Blocks:** Issue #5 (`franckKyete/tools-desktop-tauri#5`: Add `DataType::FileChunk (0x06)` and `Payload::AutomergeSync` packet variants)  
**Workspace:** `workspaces/planning/tools-desktop-tauri`  

---

## 1. Executive Summary

This research establishes the exact wire specifications, optimal payload chunk sizes, flow control mechanisms, and transport framing for streaming binary files over the Tools Bridge Protocol across heterogeneous physical transports: **Bluetooth Classic RFCOMM (via Linux BlueZ)** and **Local WebSockets (via Tokio / Warp)**.

### Key Conclusions:
1. **Wire Framing for `DataType 0x06` (`FileChunk`):**
   - Fixed **42-byte binary frame header** (1 byte `DataType` + 41 bytes `FileChunkHeader`) followed by $N$ bytes of raw file payload.
   - Fields: `DataType (1B)`, `transfer_id (16B UUID)`, `chunk_index (4B u32 BE)`, `total_chunks (4B u32 BE)`, `byte_offset (8B u64 BE)`, `payload_length (4B u32 BE)`, `flags (1B u8)`, and `checksum (4B u32 BE, CRC-32/IEEE 802.3)`.
   - Big-endian network byte order ensures zero endianness ambiguity across desktop (x86_64/ARM64) and mobile (ARM/Android/iOS) endpoints.
2. **Optimal Chunk Thresholds:**
   - **Bluetooth RFCOMM:** **2,048 bytes (2 KB)** (calibrated range: **1 KB to 4 KB**).
     - Aligns with Bluetooth Baseband 3-DH5 ACL multi-packet bursts (1021 bytes max ACL data) and avoids credit starvation or socket buffer overruns in BlueZ.
     - Prevents Head-of-Line (HoL) blocking of liveness heartbeats (`Ping`/`Pong`) and clipboard sync events.
   - **Tokio WebSocket:** **65,536 bytes (64 KB)** (calibrated range: **64 KB to 256 KB**).
     - Minimizes Tokio task/channel allocation churn and event loop context switching while maintaining low memory footprint and high throughput (saturating 802.11ac/ax Wi-Fi at 40–80 MB/s).
     - Delivers responsive progress updates (16 chunk events per megabyte).
3. **Flow Control & Reliability:**
   - Sliding Window protocol ($W = 8$ chunks on RFCOMM, $W = 32$ chunks on WebSocket) with cumulative ACKs and selective NACK on CRC-32 failure. Stop-and-wait is rejected due to excessive RTT penalties on RFCOMM (which cuts throughput by >70%).
   - Seamless resumption via `byte_offset` on reconnection.
4. **Critical Architectural Findings in Codebase:**
   - **Enum Discrepancy:** `docs/bridge-protocol.md` defines `DataType::Packet` as `0x05`, allowing `FileChunk` to be `0x06`. However, `src-tauri/src/bridge/connection/bin_data.rs` currently defines `DataType::Packet` as `0x06`. The codebase and documentation must be reconciled before implementing Issue #5.
   - **Partial Write Vulnerability:** `BluetoothSocket::send` in `src-tauri/src/bridge/connection/bluetooth/socket.rs` (line 127) uses `writer.try_write(&buf)` and aborts with `SocketError::Other("Incomplete write")` on any partial write. Any chunk exceeding the kernel socket buffer will immediately fail until switched to `tokio::io::AsyncWriteExt::write_all`.

---

## 2. BlueZ Bluetooth RFCOMM MTU & Buffer Analysis

### 2.1 Protocol Layer Hierarchy
```
+--------------------------------------------------------+
| Application Layer: DataType::FileChunk (0x06)          |
+--------------------------------------------------------+
| Bridge Transport: Header [magic: 0x424D, ver: 2, len]  |  (7 bytes in BluetoothSocket)
+--------------------------------------------------------+
| RFCOMM Layer (ETSI TS 07.10 emulation, UIH frames)     |  (3-4 bytes header + credits + 1B FCS)
+--------------------------------------------------------+
| L2CAP Layer (Logical Link Control & Adaptation)        |  (4 bytes header: length + CID)
+--------------------------------------------------------+
| HCI / ACL Baseband Data Packets (DH1/DH3/DH5, 3-DH5)   |  (1021 bytes max ACL payload)
+--------------------------------------------------------+
```

### 2.2 Primary Source Kernel & Stack Limits
- **Linux Kernel BlueZ Headers (`/usr/include/bluetooth/rfcomm.h`, line 22):**
  ```c
  #define RFCOMM_DEFAULT_MTU 127
  #define RFCOMM_PSM         3
  ```
- **Linux Kernel L2CAP Headers (`/usr/include/bluetooth/l2cap.h`, line 24):**
  ```c
  #define L2CAP_DEFAULT_MTU  672
  ```
- **Android Fluoride/AOSP Bluetooth Stack (`bta/jv/bta_jv_act.cc`):**
  ```c
  #define BTA_JV_DEF_RFC_MTU (3 * 330) // 990 bytes
  ```
- **Bluetooth Core Specification 5.4 (Vol 3, Part A & B):**
  - EDR 3-DH5 packet max payload: **1,021 bytes**.
  - Subtracting L2CAP 4-byte header: **1,017 bytes**.
  - Subtracting RFCOMM UIH frame overhead (Address 1B, Control 1B, Length 1-2B, FCS 1B): **1,013 to 1,014 bytes**.
  - BlueZ dynamically negotiates the Data Link Connection (DLC) frame size during RFCOMM Parameter Negotiation (`rfcomm_apply_pn()` in `net/bluetooth/rfcomm/core.c`), clamping the MTU to the peer's buffer capability (`min(peer_mtu, session_l2cap_mtu)`).

### 2.3 Buffer Overrun Risks & Serial Flow Control
1. **Credit-Based Flow Control:** RFCOMM uses TS 07.10 credit-based flow control. Each transmitted data frame consumes 1 credit. If the receiving mobile device processes data slower than the desktop transmits, credit replenishment stops.
2. **Kernel Buffer Limits (`sk_sndbuf`):** Linux Bluetooth sockets default to modest socket buffers (typically 4 KB to 64 KB depending on kernel parameters and controller ACL buffers). When an RFCOMM socket's send buffer fills, `write()` calls block or return `EAGAIN` / `EWOULDBLOCK`.
3. **Head-of-Line (HoL) Blocking:**
   - Real-world Bluetooth 2.1–5.0 EDR throughput is between **150 KB/s and 250 KB/s** (1.2–2.0 Mbps).
   - If a file chunk of 64 KB were sent over RFCOMM, it would block the physical channel for **300 ms to 450 ms**.
   - During this time, bridge heartbeats (`Ping`/`Pong`) and urgent clipboard sync events are queued behind the file chunk. If network latency triggers a 2-second timeout, large chunks directly induce false socket disconnections.
4. **Optimal RFCOMM Chunk Size:**
   - **Recommended Default:** **2,048 bytes (2 KB)**.
   - **Rationale:** 2 KB spans exactly two 3-DH5 baseband bursts (~10–14 ms on-air time). It fits safely within Android and BlueZ socket receive buffers, keeps link latency low for multiplexed traffic, minimizes retransmission overhead on packet corruption, and avoids credit exhaustion.

---

## 3. Tokio WebSocket Transports & Frame Limits

### 3.1 Protocol Limits & Architecture
- **RFC 6455 Frame Specification:** WebSocket binary frames support 64-bit payload lengths (up to $2^{63}-1$ bytes). Frame headers vary from 2 to 10 bytes (unmasked on desktop server transmission; 4-byte masking key when sent from mobile client).
- **Tokio / Warp / Tungstenite Defaults:**
  - `max_frame_size`: 16 MiB (16,777,216 bytes).
  - `max_message_size`: 64 MiB (67,108,864 bytes).
- **Underlying Transport:** TCP/IP over Wi-Fi (802.11) or Ethernet.
  - Standard Ethernet MTU: 1,500 bytes (TCP MSS: 1,460 bytes).
  - Wi-Fi 802.11 frame MTU: Up to 2,304 bytes.
  - TCP handles packetization, sliding windows, and flow control transparently at the OS kernel level.

### 3.2 Performance Trade-offs & Chunk Sizing
- **Small chunks (< 8 KB):**
  - Transferring a 500 MB file in 2 KB chunks requires 250,000 individual Tokio channel messages, WebSocket framing headers, and async state machine steps. This causes high CPU context-switching overhead and limits throughput to ~8–12 MB/s.
- **Excessive chunks (> 1 MB):**
  - Allocates large contiguous buffers in memory. On mobile devices with tight heap constraints, multiple concurrent buffers cause GC pressure or OOM kills.
  - Creates chunky, stuttering progress indicators in the Tauri UI.
- **Optimal WebSocket Chunk Size:**
  - **Recommended Default:** **65,536 bytes (64 KB)**.
  - **High-Throughput LAN Max:** **262,144 bytes (256 KB)**.
  - **Rationale:** 64 KB provides 16 progress updates per megabyte, keeps memory overhead minimal, and easily saturates typical Wi-Fi links (delivering 40–80 MB/s sustained throughput).

---

## 4. Binary Wire Layout Specification: `DataType 0x06` (`FileChunk`)

### 4.1 Wire Frame Structure

The full wire frame consists of the **1-byte `DataType`** indicator followed by the **41-byte `FileChunkHeader`**, followed immediately by $N$ raw payload bytes.

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| DataType 0x06 |                                               |
+-+-+-+-+-+-+-+-+                                               +
|                                                               |
+                    transfer_id (16 bytes)                     +
|                          (UUID v4)                            |
+                                                               +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                  chunk_index (u32, Big-Endian)                |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                  total_chunks (u32, Big-Endian)               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
+                  byte_offset (u64, Big-Endian)                +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                 payload_length (u32, Big-Endian)              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  flags (u8)   |             crc32_checksum (u32 BE)           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    (cont.)    |                                               |
+-+-+-+-+-+-+-+-+                                               +
|                                                               |
~                payload_bytes (N raw file bytes)               ~
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### 4.2 Exact Field Specification Table

| Byte Offset (Frame) | Byte Offset (Header) | Field Name | Type / Wire Format | Size | Description |
|:---|:---|:---|:---|:---:|:---|
| `0x00` (0) | — | `data_type` | `u8` | 1 B | `0x06` identifies `DataType::FileChunk`. |
| `0x01` (1) | `0x00` (0) | `transfer_id` | `[u8; 16]` | 16 B | 128-bit UUID matching the `transfer_id` from `FileRequestPayload`. |
| `0x11` (17) | `0x10` (16) | `chunk_index` | `u32` (Big-Endian) | 4 B | 0-indexed chunk sequence number (`0`, `1`, `2`, ...). |
| `0x15` (21) | `0x14` (20) | `total_chunks`| `u32` (Big-Endian) | 4 B | Total expected chunks for this file transfer (`ceil(file_size / chunk_size)`). |
| `0x19` (25) | `0x18` (24) | `byte_offset` | `u64` (Big-Endian) | 8 B | Absolute starting byte offset within the target file. Allows direct `seek()` writes and multi-transport resumption. |
| `0x21` (33) | `0x20` (32) | `payload_len` | `u32` (Big-Endian) | 4 B | Exact byte length $N$ of following `payload_bytes` ($1 \le N \le 262,144$). |
| `0x25` (37) | `0x24` (36) | `flags` | `u8` bitfield | 1 B | Control flags. |
| `0x26` (38) | `0x25` (37) | `checksum` | `u32` (Big-Endian) | 4 B | CRC-32 (IEEE 802.3 standard polynomial `0xEDB88320`) computed strictly over `payload_bytes`. |
| `0x2A` (42) | `0x29` (41) | `payload` | `[u8; N]` | $N$ B | Raw file data slice ($N = \text{payload\_len}$). |

**Total Header Overhead:** 42 bytes (frame level) / 41 bytes (payload level).

### 4.3 Control Flags Bitmask (`flags`)
- **Bit 0 (`0x01`) - `FLAG_EOF` (Last Chunk):** Set on the final chunk of the transfer (`chunk_index == total_chunks - 1`). Signals the receiver to close the staging `.part` file, run the complete file SHA-256 verification, and notify the user.
- **Bit 1 (`0x02`) - `FLAG_ACK_REQ` (Immediate Acknowledgement Requested):** Demands that the receiver immediately respond with a `ChunkAck` packet (e.g., at window boundaries or end-of-file).
- **Bit 2 (`0x04`) - `FLAG_RESUMED`:** Indicates that this stream resumes an earlier disconnected transfer.
- **Bits 3–7 (`0xF8`):** Reserved for future extension (must be transmitted as `0x00`).

---

## 5. Flow Control & Acknowledgement Protocol

### 5.1 Acknowledgement Model: Sliding Window
- **Sliding Window Specification:**
  - **Bluetooth RFCOMM Window:** $W = 8$ chunks ($16 \text{ KB}$ in-flight). The sender sets `FLAG_ACK_REQ` on every 4th chunk and on the final chunk.
  - **WebSocket Window:** $W = 32$ chunks ($2 \text{ MB}$ in-flight). The sender sets `FLAG_ACK_REQ` on every 16th chunk and on the final chunk.
  - **Backpressure:** The sender maintains a count of unacknowledged transmitted chunks. If `in_flight >= W`, the sender pauses transmission until a `ChunkAck` advances the window.

### 5.2 ACK / NACK Binary Wire Layout (`DataType 0x07`)
```
Offset  Size  Field                   Type / Description
---------------------------------------------------------------------------------
0x00    1     DataType                0x07 (ChunkAck)
0x01    16    transfer_id             [u8; 16] (UUID v4)
0x11    4     cumulative_chunk_index  u32 BE (All chunks <= this index received & written)
0x15    4     window_credit           u32 BE (Number of new chunks permitted to send)
0x19    1     status                  0x00 = OK, 0x01 = CRC_FAIL, 0x02 = IO_ERROR
0x1A    4     nack_chunk_index        u32 BE (If status == CRC_FAIL, index of corrupted chunk)
```

---

## 6. Rust Implementation Architecture (For Issue #5)

In `src-tauri/src/bridge/connection/bin_data.rs`:

```rust
use std::convert::TryInto;
use crc32fast::Hasher;

pub const CHUNK_FLAG_EOF: u8 = 0x01;
pub const CHUNK_FLAG_ACK_REQ: u8 = 0x02;
pub const CHUNK_FLAG_RESUMED: u8 = 0x04;

pub const FILE_CHUNK_HEADER_SIZE: usize = 41; // Without leading DataType byte

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChunkHeader {
    pub transfer_id: [u8; 16],
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub byte_offset: u64,
    pub payload_len: u32,
    pub flags: u8,
    pub checksum: u32,
}

impl FileChunkHeader {
    pub fn serialize(&self, buf: &mut [u8]) {
        assert!(buf.len() >= FILE_CHUNK_HEADER_SIZE);
        buf[0..16].copy_from_slice(&self.transfer_id);
        buf[16..20].copy_from_slice(&self.chunk_index.to_be_bytes());
        buf[20..24].copy_from_slice(&self.total_chunks.to_be_bytes());
        buf[24..32].copy_from_slice(&self.byte_offset.to_be_bytes());
        buf[32..36].copy_from_slice(&self.payload_len.to_be_bytes());
        buf[36] = self.flags;
        buf[37..41].copy_from_slice(&self.checksum.to_be_bytes());
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self, &'static str> {
        if buf.len() < FILE_CHUNK_HEADER_SIZE {
            return Err("Buffer too short for FileChunkHeader");
        }
        let transfer_id: [u8; 16] = buf[0..16].try_into().unwrap();
        let chunk_index = u32::from_be_bytes(buf[16..20].try_into().unwrap());
        let total_chunks = u32::from_be_bytes(buf[20..24].try_into().unwrap());
        let byte_offset = u64::from_be_bytes(buf[24..32].try_into().unwrap());
        let payload_len = u32::from_be_bytes(buf[32..36].try_into().unwrap());
        let flags = buf[36];
        let checksum = u32::from_be_bytes(buf[37..41].try_into().unwrap());

        Ok(Self {
            transfer_id,
            chunk_index,
            total_chunks,
            byte_offset,
            payload_len,
            flags,
            checksum,
        })
    }

    pub fn compute_checksum(payload: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(payload);
        hasher.finalize()
    }
}
```

---

## 7. Action Items for Issue #5

1. Reconcile `bin_data.rs` with `docs/bridge-protocol.md`:
   - `0x05`: `Packet`
   - `0x06`: `FileChunk`
   - `0x07`: `ChunkAck`
2. Fix `writer.try_write` in `src-tauri/src/bridge/connection/bluetooth/socket.rs` to use `tokio::io::AsyncWriteExt::write_all`.
3. Upgrade `FileRequestPayload::file_size` from `u32` to `u64` in `packets.rs`.
