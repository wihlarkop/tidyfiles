# TidyFiles

<div align="center">

**Automatically organize your files with custom rules and smart features**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri-24c8db)](https://tauri.app/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-ff3e00)](https://svelte.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange)](https://www.rust-lang.org/)

</div>

## 📖 Overview

TidyFiles is a powerful desktop application that helps you automatically organize files based on custom rules. Whether you need to sort documents by date, organize photos by location, or categorize downloads by file type, TidyFiles makes it simple and efficient.

## ✨ Features

### Core Organization
- **🎯 Rule-Based Organization**: Create custom rules with flexible conditions
  - Match by filename, extension, size, creation/modification date
  - Support for regex patterns, contains, starts with, ends with
  - Combine multiple conditions with AND/OR logic
  - Set rule priorities for execution order

- **📋 Smart Placeholders**: Dynamic file naming with metadata
  - `{filename}`, `{ext}`, `{name}` - Basic file info
  - `{parent}` - Parent folder name
  - `{size}` - Human-readable file size
  - `{year}`, `{month}`, `{day}` - Modified date parts
  - `{modified_date}`, `{created_date}` - Full dates (YYYY-MM-DD)
  - `{created_year}`, `{created_month}`, `{created_day}` - Created date parts

- **🔍 Advanced Filtering**: Find exactly what you need
  - Search files by name or path
  - Filter by extension
  - Filter preview by rule
  - Pagination for large file lists

### Smart Features
- **✅ Batch Selection**: Select specific files to organize with checkboxes
- **🧪 Dry Run Mode**: Test your rules before making changes
  - Simulate operations without moving files
  - Preview exactly what would happen
  - Identify potential conflicts before they occur

- **🔄 Duplicate Finder**: Identify and manage duplicate files
  - Find duplicates by content hash
  - Configurable minimum file size
  - Group duplicates for easy review
  - Batch delete unwanted duplicates

- **📜 Operation History**: Complete audit trail
  - Track all file operations
  - See source and destination paths
  - Roll back operations if needed
  - Search and filter history

### User Experience
- **⏳ Skeleton Loading Screens**: Smooth loading experience
- **🎨 Dark/Light Theme**: Comfortable viewing in any environment
- **⚙️ Configurable Settings**: Customize default behavior
  - Scan defaults (extensions, depth, hidden files)
  - UI preferences (items per page, theme)
  - Duplicate detection settings
- **⌨️ Keyboard Shortcuts**: Efficient navigation and actions
- **🔔 Smart Notifications**: Stay informed with toast messages
- **📊 Progress Indicators**: Real-time feedback on operations

## 🛠️ Tech Stack

- **Frontend**: Svelte 5 (with Runes) + SvelteKit
- **Backend**: Rust + Tauri 2.0
- **Styling**: Tailwind CSS 4
- **Build Tool**: Vite 6

## 📥 Installation

### Download Pre-built Binaries

Download the latest installer for your platform from the [Releases](https://github.com/wihlarkop/tidyfiles/releases) page:

- **Windows**: `TidyFiles_x.x.x_x64_setup.exe` or `.msi`
- **macOS**: `TidyFiles_x.x.x_x64.dmg` or `TidyFiles_x.x.x_aarch64.dmg`
- **Linux**: `TidyFiles_x.x.x_amd64.deb`, `.rpm`, or `.AppImage`

### Build from Source

#### Prerequisites

- **Node.js** 18+ and **pnpm** 10+
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **System dependencies**:
  - **Windows**: No additional dependencies required
  - **macOS**: Xcode Command Line Tools
  - **Linux**:
    ```bash
    # Ubuntu/Debian
    sudo apt install libwebkit2gtk-4.1-dev \
      build-essential curl wget file libssl-dev \
      libgtk-3-dev libayatana-appindicator3-dev \
      librsvg2-dev

    # Fedora
    sudo dnf install webkit2gtk4.1-devel \
      openssl-devel curl wget file \
      gtk3-devel libappindicator-gtk3-devel \
      librsvg2-devel

    # Arch
    sudo pacman -S webkit2gtk-4.1 \
      base-devel curl wget file \
      openssl gtk3 libappindicator-gtk3 \
      librsvg
    ```

#### Clone and Build

```bash
# Clone the repository
git clone git@github.com:wihlarkop/tidyfiles.git
cd tidyfiles

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/`

## 🚀 Building Production Installers

### Windows
```bash
pnpm tauri build
```
**Output**:
- `src-tauri/target/release/bundle/msi/TidyFiles_x.x.x_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/TidyFiles_x.x.x_x64-setup.exe`

### macOS
```bash
pnpm tauri build
```
**Output**:
- `src-tauri/target/release/bundle/dmg/TidyFiles_x.x.x_x64.dmg`
- `src-tauri/target/release/bundle/macos/TidyFiles.app`

### Linux
```bash
pnpm tauri build
```
**Output**:
- `src-tauri/target/release/bundle/deb/tidyfiles_x.x.x_amd64.deb`
- `src-tauri/target/release/bundle/rpm/tidyfiles-x.x.x-1.x86_64.rpm`
- `src-tauri/target/release/bundle/appimage/tidyfiles_x.x.x_amd64.AppImage`

## 📚 Usage Guide

### 1. Scan Files

1. Click **"Browse"** to select a folder to organize
2. Configure scan options (optional):
   - Include hidden files
   - Follow symbolic links
   - Maximum depth
   - File extensions filter
3. Click **"Start Scan"** to analyze the folder

### 2. Create Rules

1. Click **"+ Add Rule"** to create an organization rule
2. Set the destination folder (use placeholders for dynamic paths)
3. Optional: Add a rename pattern
4. Add conditions to match specific files:
   - Field: Filename, Extension, File Size, Created, Modified
   - Operator: Equals, Contains, Starts with, Ends with, Regex, etc.
   - Value: The pattern to match
5. Set priority (lower numbers execute first)

**Example Rule**:
- **Name**: "Organize PDFs by Year"
- **Destination**: `C:/Documents/PDFs/{year}`
- **Conditions**: Extension equals "pdf"

### 3. Preview Changes

1. Click **"Preview Rules"** to see which files match
2. Use search and filter to review matches
3. Select specific files with checkboxes (optional)
4. Enable **Dry Run Mode** to test without moving files

### 4. Organize

1. Click **"Start Organization"** (or **"Test Organization"** in dry run mode)
2. Review results
3. Check operation history for complete audit trail
4. Roll back if needed

## 🔧 Development

### Project Structure

```
tidyfiles/
├── src/                    # Frontend (Svelte)
│   ├── lib/
│   │   ├── api/           # Tauri command bindings
│   │   ├── components/    # Reusable Svelte components
│   │   ├── stores/        # Svelte stores
│   │   ├── types/         # TypeScript types
│   │   └── utils/         # Utility functions
│   └── routes/            # SvelteKit routes
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── commands/      # Tauri commands
│   │   ├── engine/        # Core logic
│   │   ├── models/        # Data models
│   │   └── storage/       # Data persistence
│   └── icons/             # Application icons
└── static/                # Static assets
```

### Available Scripts

```bash
# Development
pnpm dev                    # Run dev server (frontend only)
pnpm tauri dev             # Run full app with hot reload

# Type checking
pnpm check                 # Check TypeScript/Svelte types
pnpm check:watch          # Watch mode

# Building
pnpm build                 # Build frontend
pnpm tauri build          # Build complete app with installers

# Rust
cd src-tauri
cargo check               # Check Rust code
cargo test                # Run tests
cargo clippy              # Lint Rust code
```

### Adding New Features

1. **Frontend**: Add components in `src/lib/components/`
2. **Backend**: Add Rust functions in `src-tauri/src/commands/`
3. **Bindings**: Export commands in `src-tauri/src/lib.rs`
4. **Types**: Update TypeScript types in `src/lib/types/`

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Coding Standards

- **Frontend**: Follow Svelte/TypeScript best practices
- **Backend**: Follow Rust conventions (run `cargo fmt` and `cargo clippy`)
- **Commits**: Use conventional commits (feat:, fix:, docs:, etc.)

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

