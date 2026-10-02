# Desktop Development Guide

This guide provides instructions for setting up the development environment, running, and debugging **Tools Desktop** (`tools-desktop-tauri`).

---

## 1. Prerequisites

### Systems Dependencies (Linux / Ubuntu / Arch)
- **Rust Toolchain**: `rustup` with stable Rust (`rustc 1.80+`).
- **JavaScript Runtime & Package Manager**: [Bun](https://bun.sh/) (v1.2+) or Node.js (v20+).
- **Desktop Libraries (Linux)**:
  - Ubuntu/Debian:
    ```bash
    sudo apt install build-essential pkg-config libdbus-1-dev libglib2.0-dev libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
    ```
  - Arch Linux:
    ```bash
    sudo pacman -S base-devel pkgconf dbus glib2 openssl bluez bluez-utils
    ```
- **Bluetooth Subsystem**: `bluez` service active and running:
  ```bash
  sudo systemctl start bluetooth
  ```

---

## 2. Getting Started

### Installation
From the desktop directory (`tools-desktop-tauri`):
```bash
bun install
```

### Running Development Server
Launch the Tauri development environment (starts Vite dev server and compiles Rust backend in debug mode):
```bash
bun run tauri dev
```

### Type Checking & Linting
```bash
bun run tsc --noEmit
```

### Building for Release
```bash
bun run tauri build
```

---

## 3. Architecture & Directory Overview

```
tools-desktop-tauri/
├── src-tauri/                  # Rust backend
│   ├── Cargo.toml              # Rust crate dependencies
│   ├── src/
│   │   ├── main.rs             # Application entrypoint
│   │   ├── lib.rs              # Tauri setup, IPC commands, lifecycle hooks
│   │   ├── bridge/             # Bluetooth & WebSocket bridge system
│   │   │   ├── packets.rs      # Wire protocol packets and serialization
│   │   │   ├── bridge.rs       # Bridge instance management
│   │   │   └── connection/     # Bluetooth (BlueZ) and WebSocket transports
│   │   ├── clipboard/          # Background arboard clipboard daemon
│   │   ├── notes/              # Notes data structures & manager
│   │   ├── storage/            # SQLite engine & Postcard serialization
│   │   └── event/              # Asynchronous event manager
├── src/                        # React 19 Frontend
│   ├── routes/                 # TanStack file-based routes
│   │   ├── index.tsx           # Home / Welcome route
│   │   ├── setup.tsx           # Pairing QR code generator
│   │   ├── notes.tsx           # Notes masonry grid
│   │   └── note.{-$noteId}.tsx # BlockNote rich text editor
│   ├── components/             # Reusable UI components
│   └── styles/                 # Tailwind CSS styles
```

---

## 4. Debugging & Diagnostics

### Verbose Logging
The Rust backend utilizes `env_logger`. Run with `RUST_LOG=debug` to inspect bridge connection negotiation and frame dispatch:
```bash
RUST_LOG=debug bun run tauri dev
```

### Testing WebSocket Transport Directly
If Bluetooth is disabled or unavailable, the desktop launches a WebSocket server. Check the advertisement JSON generated at `/setup` to retrieve the local WebSocket URL (e.g. `ws://192.168.1.150:8080`), and connect using `websocat`:
```bash
websocat ws://127.0.0.1:8080
```

### Hyprland Integration
Under Hyprland, the application dynamically requests floating window placement in `lib.rs`:
```rust
Keyword::set("windowrulev2", "float , class:^(tools-desktop-tauri)")?;
Keyword::set("windowrulev2", format!("move {to_x} {to_y}, class:^(tools-desktop-tauri)"))?;
```
If using a different window manager (GNOME, KDE, i3), these calls fail gracefully without terminating the application.
