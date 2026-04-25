# Telegram Desktop

A Telegram client built with Tauri 2, React, TypeScript, and tdlib.

## Features

- Login with phone number and verification code
- View chat list
- Send and receive messages
- Cross-platform desktop app (Windows, macOS, Linux)

## Prerequisites

- Node.js 20+
- Rust 1.70+
- Telegram API credentials (get from https://my.telegram.org)

## Setup

1. Clone the repository
2. Install dependencies:
```bash
npm install
```

3. Run in development mode:
```bash
npm run tauri dev
```

4. Build for production:
```bash
npm run tauri build
```

## GitHub Actions

The project includes CI/CD workflows for building on multiple platforms:
- Windows (MSI, NSIS)
- macOS (DMG, App)
- Linux (AppImage, deb, rpm)

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri 2 (Rust)
- **TDLib**: tdlib for Telegram API

## Note

The current implementation uses placeholder tdlib integration. To fully integrate tdlib:
1. Download precompiled tdlib from https://github.com/tdlib/td/releases
2. Or build tdlib from source
3. Update the Rust code to use tdlib-sys or tdlib-rs

## License

MIT