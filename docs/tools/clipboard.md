# Desktop Clipboard Sync Tool

The **Desktop Clipboard Sync Tool** integrates with the system clipboard to provide seamless bi-directional synchronization with mobile companions with automated echo cancellation.

---

## Architecture

```mermaid
sequenceDiagram
    autonumber
    participant LocalClip as System Clipboard (OS)
    participant Monitor as Desktop Clipboard Monitor
    participant Filter as Echo Filter (Dual Ring Buffers)
    participant Bridge as Bridge Transport
    participant Mobile as Mobile Companion

    Note over LocalClip,Monitor: User copies text on Desktop
    LocalClip->>Monitor: Clipboard Changed Event / Poll
    Monitor->>Filter: Check SHA-256(content) in recent_applied_hashes
    Note over Filter: Hash not found -> Local edit!
    Filter->>Filter: Push SHA-256 to recent_sent_hashes
    Monitor->>Bridge: Send Packet(ClipboardPayload: origin_device_id, clip_id, content)
    Bridge->>Mobile: Transmit Packet

    Note over Mobile,Bridge: Remote copy arrives from Mobile
    Mobile->>Bridge: Receive Packet(ClipboardPayload)
    Bridge->>Filter: Push SHA-256 to recent_applied_hashes
    Bridge->>LocalClip: arboard::Clipboard.set_text(content)
    Note over LocalClip,Monitor: OS clipboard triggers monitor
    Monitor->>Filter: Check SHA-256 in recent_applied_hashes
    Note over Filter: Hash matches! Echo loop cancelled.
```

---

## 1. Echo Cancellation & Provenance Tracking

To prevent infinite ping-pong feedback loops when synchronizing clipboards across companions:

### Dual Circular Hash Ring Buffers
`ClipboardEchoFilter` maintains two fixed-capacity circular buffers (capacity = 20):
1. **`recent_applied_hashes`**: Stores SHA-256 hashes of text received from remote peers and written to the local clipboard.
2. **`recent_sent_hashes`**: Stores SHA-256 hashes of local copies broadcast to peers.

### Broadcast Suppression
When the local system clipboard monitor detects changed text:
- Computes `hash = sha256(text)`.
- If `hash` exists in `recent_applied_hashes`, it was written by a remote peer; the event is dropped silently.
- Otherwise, `hash` is pushed to `recent_sent_hashes` and broadcast to connected peers.
- Single packet cap: payload text larger than 64 KB is dropped to avoid saturating Bluetooth serial links.

---

## 2. Wire Schema (`ClipboardPayload`)

```rust
pub struct ClipboardPayload {
    pub content: String,
    pub origin_device_id: String,
    pub clip_id: String,
}
```
