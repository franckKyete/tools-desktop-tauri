# Tools Desktop

**Tools Desktop** (`tools-desktop-tauri`) is the workstation companion and hub application of the **Tools** ecosystem, built with **Tauri v2**, **Rust**, and **React 19**.

"Tools" is an ever-evolving collection of utilities seamlessly bridging PC and mobile devices over local Bluetooth and WebSocket links without third-party servers.

---

## Features

- **Local-First Bridge**: Bi-directional communication across devices over Bluetooth RFCOMM (via Linux BlueZ) and local LAN WebSockets.
- **Frictionless Pairing**: Instant QR-code advertisement and session resumption tokens.
- **Notes Editor**: Rich-text block-based note-taking powered by BlockNote and local SQLite with Postcard serialization.
- **Clipboard Sync**: Automatic, real-time clipboard synchronization powered by `arboard`.
- **File Transfer**: Direct peer-to-peer file transfer protocol without cloud intermediaries.
- **Hyprland Integration**: Automatic floating window positioning and system tray support.

---

## Documentation

Full project documentation is available in the [`docs/`](./docs/README.md) folder:

- **[Documentation Index](./docs/README.md)**
- **[Vision & Roadmap](./docs/vision-and-roadmap.md)**
- **[Bridge Protocol Specification](./docs/bridge-protocol.md)**
- **[System Architecture](./docs/system-architecture.md)**
- **Tools**:
  - [Notes Tool](./docs/tools/notes.md)
  - [Clipboard Tool](./docs/tools/clipboard.md)
  - [File Transfer](./docs/tools/file-transfer.md)
  - [Future Tools Roadmap](./docs/tools/future-tools.md)
- **Guides**:
  - [Development Guide](./docs/guides/development.md)
  - [Adding a New Tool](./docs/guides/adding-a-tool.md)

---

## Quick Start

### Prerequisites
- [Bun](https://bun.sh/) (v1.2+) or Node.js (v20+)
- Rust (v1.80+)
- Linux system dependencies: `pkg-config`, `libdbus-1-dev`, `libglib2.0-dev`, `bluez`

### Run Development
```bash
bun install
bun run tauri dev
```

### Build Release
```bash
bun run tauri build
```
