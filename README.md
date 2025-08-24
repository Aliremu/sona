# 🎸 Sona

A modern desktop application for efficient guitar practicing, built with Rust and React in Tauri.

## ✨ Features

- **🎥 Video Playback**: Integrated YouTube player with speed control (0.1x - 2.0x)
- **🎛️ VST Plugin Support**: Load and manage audio effects plugins
- **🎵 Playlist Management**: Organize your practice songs and exercises

## 📝 TODO
- [ ] Download YouTube videos / upload videos
- [ ] Ability to change playback speed / pitch shift
- [ ] Integrate sheet music support
- [ ] Auto sync music with sheet music
- [ ] Scrape sheet music from online sources / parse guitar pro tabs
- [ ] Node graph based plugin routing
- [ ] Refactor VST3 library
- [ ] Refactor Audio library
- [ ] SQLite for managing playlists, and practice stats

## 🛠️ Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri
- **UI**: Tailwind CSS + shadcn/ui components
- **Audio**: VST3 plugin system
- **Video**: YouTube Player API integration

## 🚀 Getting Started

### Prerequisites

- Node.js (v16 or higher)
- Rust (latest stable)
- pnpm (recommended) or npm

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/Aliremu/sona.git
   cd sona
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Start the development server:
   ```bash
   pnpm tauri dev
   ```

### Building for Production

```bash
pnpm tauri build
```