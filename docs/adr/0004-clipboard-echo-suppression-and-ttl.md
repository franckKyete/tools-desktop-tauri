# ADR-0004: Clipboard Echo Suppression with Dual Ring Buffers and 5-Minute TTL

## Context
Bi-directional clipboard synchronization between desktop and mobile companions causes infinite feedback loops if incoming clipboard updates are re-broadcast as local copy events. Furthermore, offline copies can overwhelm peers with obsolete intermediate clippings upon reconnecting.

## Decision
We extended `ClipboardPayload` with provenance metadata: `origin_device_id` and `clip_id`. We implemented dual 20-entry circular SHA-256 hash ring buffers:
- `recent_applied_hashes`: stores hashes of remote text applied to the local clipboard.
- `recent_sent_hashes`: stores hashes of local text broadcast to peers.

When the local clipboard changes, the text hash is checked against `recent_applied_hashes`. If present, the change was caused by a remote peer and outbound broadcast is dropped. If absent, the hash is pushed to `recent_sent_hashes` and broadcast.

When offline, mobile stores only the single latest clipboard snapshot with a 5-minute time-to-live (TTL). Older copies are discarded, and only the latest copy is transmitted on reconnection.

## Consequences
- Total elimination of clipboard ping-pong storms across connected peers.
- Clean user experience upon reconnecting without flashing outdated intermediate clippings.
- Bounded memory overhead (20 SHA-256 strings per buffer).
