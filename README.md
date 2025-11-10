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

### System Requirements

#### Windows
- **Operating System**: Windows 10 or later (64-bit)
- **Dependencies**: WebView2 Runtime (automatically installed with the app)
- **Disk Space**: ~30 MB
- **RAM**: 4 GB minimum, 8 GB recommended

#### macOS
- **Operating System**: macOS 10.13 (High Sierra) or later
- **Architecture**: Intel (x86_64) or Apple Silicon (ARM64)
- **Disk Space**: ~40 MB
- **RAM**: 4 GB minimum, 8 GB recommended

#### Linux
- **Operating System**: Modern Linux distribution with glibc 2.31+
- **Desktop Environment**: Any (GNOME, KDE, XFCE, etc.)
- **Display Server**: X11 or Wayland
- **Disk Space**: ~35 MB
- **RAM**: 4 GB minimum, 8 GB recommended

**Required Dependencies:**

| Distribution | Required Packages |
|--------------|------------------|
| **Arch / Manjaro** | `webkit2gtk gtk3` |
| **Debian / Ubuntu** | `libwebkit2gtk-4.1-0 libayatana-appindicator3-1` |
| **Fedora / RHEL** | `webkit2gtk4.1` |
| **Other** | Use AppImage (no dependencies needed) |

### Download Pre-built Binaries

Download the latest installer from the [Releases](https://github.com/wihlarkop/tidyfiles/releases) page:

#### Windows
- **NSIS Setup** (recommended): `TidyFiles_x.x.x_x64-setup.exe`
- **MSI Installer** (for IT/enterprise): `TidyFiles_x.x.x_x64_en-US.msi`

**Installation:**
1. Download the setup file
2. Run the installer
3. Follow the installation wizard
4. Note: Windows SmartScreen may show a warning - click "More info" → "Run anyway"

#### macOS
- **Universal DMG**: `TidyFiles_x.x.x_universal.dmg` (works on both Intel and Apple Silicon)

**Installation:**
1. Download the DMG file
2. Open the DMG
3. Drag TidyFiles to your Applications folder
4. **IMPORTANT:** On first launch, you'll see a security warning - follow the steps below

**⚠️ macOS Security Warning (Gatekeeper)**

Since TidyFiles is not code-signed with an Apple Developer certificate, macOS will block it from opening. This is normal and safe. Choose one of these methods:

**Method 1: Automated Helper Script (Easiest)**
```bash
# Download and run the installation helper
curl -O https://raw.githubusercontent.com/wihlarkop/tidyfiles/main/install-macos.sh
bash install-macos.sh
```

**Method 2: Right-click to Open**
1. Find TidyFiles in your Applications folder
2. **Right-click** (or Control+click) on TidyFiles.app
3. Select **"Open"** from the context menu
4. Click **"Open"** in the dialog that appears

**Method 3: Remove quarantine attribute**
```bash
xattr -cr /Applications/TidyFiles.app
```

**Method 4: System Settings**
1. Try to open TidyFiles normally (it will be blocked)
2. Go to **System Settings** → **Privacy & Security**
3. Scroll down to find a message about TidyFiles
4. Click **"Open Anyway"**

After using any of these methods once, macOS will remember your choice and you can open TidyFiles normally.

#### Linux

**For Arch Linux / Manjaro (Recommended):**
```bash
# 1. Install dependencies
sudo pacman -S webkit2gtk gtk3

# 2. Download and install package
wget https://github.com/wihlarkop/tidyfiles/releases/latest/download/tidyfiles-x.x.x-1-x86_64.pkg.tar.zst
sudo pacman -U tidyfiles-x.x.x-1-x86_64.pkg.tar.zst
```

**For Debian / Ubuntu:**
```bash
# Download and install
wget https://github.com/wihlarkop/tidyfiles/releases/latest/download/TidyFiles_x.x.x_amd64.deb
sudo dpkg -i TidyFiles_x.x.x_amd64.deb

# If you get dependency errors:
sudo apt-get install -f
```

**For Any Linux Distribution (AppImage):**
```bash
# Download
wget https://github.com/wihlarkop/tidyfiles/releases/latest/download/TidyFiles_x.x.x_amd64.AppImage

# Make executable
chmod +x TidyFiles_x.x.x_amd64.AppImage

# Run
./TidyFiles_x.x.x_amd64.AppImage
```

**Optional: Add to Application Menu**
```bash
# Move to /opt
sudo mv TidyFiles_x.x.x_amd64.AppImage /opt/tidyfiles

# Create desktop entry
cat > ~/.local/share/applications/tidyfiles.desktop <<EOF
[Desktop Entry]
Name=TidyFiles
Exec=/opt/tidyfiles
Type=Application
Categories=Utility;FileTools;
Icon=tidyfiles
Terminal=false
EOF
```

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

## Best Practices for Using TidyFiles

### Safety First
1. **Always use Dry Run Mode first** - Test your rules before applying them to ensure they work as expected
2. **Start with a test folder** - Try organizing a small folder before applying rules to important files
3. **Back up important files** - While TidyFiles has undo functionality, it's always good to have backups
4. **Review the preview** - Check which files match your rules before organizing

### Organizing Tips
1. **Use descriptive rule names** - Makes it easier to manage multiple rules
2. **Set clear priorities** - Lower numbers execute first (e.g., 1, 2, 3...)
3. **Test with small batches** - Use file selection to organize a few files at a time
4. **Check operation history** - Review what was done and undo if needed

### Performance
1. **Limit scan depth** - Set maximum depth to avoid scanning too many nested folders
2. **Filter by extension** - Only scan file types you need to organize
3. **Use batch selection** - Select only files you want to organize instead of all matches

### Dependency Installation

**Linux users:** Make sure dependencies are installed before first run:

```bash
# Arch / Manjaro
sudo pacman -S webkit2gtk gtk3

# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1

# Fedora
sudo dnf install webkit2gtk4.1
```

**If TidyFiles shows a blank screen on Linux:**
- The app will show an error message with installation instructions
- Or try running with: `GDK_BACKEND=x11 tidyfiles`

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

