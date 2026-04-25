# apm (Archuser Package Manager) 🦀

A lightweight AUR helper built in Rust. Born out of a need for a simple, fast alternative when other helpers aren't available.

## Features
- **AUR Search:** Query the official Arch User Repository RPC API.
- **Automated Builds:** Automatically clones, handles dependencies, and builds via `makepkg`.
- **Sandbox Builds:** Uses `/tmp/apm-builds` to keep your home directory clean.

## Installation

### Prerequisites
Ensure you have the base development tools and Rust installed:
```bash
sudo pacman -S --needed base-devel git rust
