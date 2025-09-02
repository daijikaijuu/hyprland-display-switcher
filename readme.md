# Hyprland Display Switcher

A modern GUI application for managing display configurations in Hyprland, built with Rust and Iced.

## Features

- **Multiple Display Modes**:
  - 🖥️ **PC Screen Only** - Use only your primary display
  - 📱 **Duplicate Displays** - Mirror content across all displays  
  - 🖼️ **Extend Displays** - Use displays as one continuous workspace
  - 📺 **Second Screen Only** - Use only your external display

- **Advanced Extended Mode Configuration**:
  - Choose primary and secondary monitors
  - Configure resolution and rotation for each display
  - Multiple layout options (Left-to-Right, Right-to-Left, Top-to-Bottom, Bottom-to-Top)
  - Automatic configuration saving and loading

- **Modern UI**:
  - Clean, dark-themed interface
  - Layer shell integration for overlay display
  - Keyboard shortcuts (ESC to exit)
  - Real-time monitor detection

## Installation

### Prerequisites

- Hyprland window manager
- Rust toolchain (for building from source)

### Building from Source

```bash
git clone https://github.com/daijikaijuu/hyprland-display-switcher.git
cd hyprland-display-switcher
cargo build --release
```

## ps
This is ai slop mostly for personal use 