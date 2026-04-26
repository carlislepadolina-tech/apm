# apm (Archuser Package Manager)

A lightweight AUR helper built in Rust. Born out of a need for a simple, fast alternative when other helpers aren't available.

## Features
- **AUR Search:** Query the official Arch User Repository RPC API.
- **Automated Builds:** Automatically clones, handles dependencies, and builds via `makepkg`.
- **Sandbox Builds:** Uses `/tmp/apm-builds` to keep your home directory clean.

## General Note
This AUR helper is built for speed and generally *not needed* to install unless your options yay or paru are not available (or youre just lazy. :P)

## Installation

### Prerequisites
Ensure you have the base development tools and Rust installed:
```bash
sudo pacman -S --needed base-devel git rust
```
Clone the repository:
```bash
git clone https://github.com/carlislepadolina-tech/apm.git
```
Build with Cargo:
```bash
cargo build --release
```
Then move to your bin, or usr/bin:
```bash
mv apm/target/release/apm /bin
# OR
mv apm/target/release/apm /usr/bin
```
## Showcase
[Screencast_20260425_222240.webm](https://github.com/user-attachments/assets/f4ae7674-69f2-406b-b3f2-6aa886231737)
## Why APM?
No bragging rights here, but apm is a great, lightweight, and fast alternative to yay or paru. Instead of ebing a feature-rich, massive, and a superior AUR helper, apm gives you the essentials to install or search the AUR for your packages.
