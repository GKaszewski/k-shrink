# Installation

## AUR (Arch Linux)

```bash
yay -S k-shrink
```

AUR page: https://aur.archlinux.org/packages/k-shrink *(placeholder — submit PKGBUILD first)*

Then enable the service:

```bash
systemctl --user enable --now k-shrink.service
```

---

## Manual (from source)

**Prerequisites:** Rust toolchain, `wayland` libraries.

```bash
git clone https://github.com/PLACEHOLDER/k-shrink
cd k-shrink
cargo build --release
```

Install the binary:

```bash
install -Dm755 target/release/k-shrink ~/.local/bin/k-shrink
```

Install the systemd service:

```bash
mkdir -p ~/.config/systemd/user
cp contrib/k-shrink.service ~/.config/systemd/user/
systemctl --user enable --now k-shrink.service
```

Install the man page (optional):

```bash
mkdir -p ~/.local/share/man/man1
cp man/k-shrink.1 ~/.local/share/man/man1/
gzip ~/.local/share/man/man1/k-shrink.1
mandb ~/.local/share/man   # update man index
```

---

## cargo install

```bash
cargo install --git https://github.com/PLACEHOLDER/k-shrink k-shrink
```

Then enable the service (uses `%h/.cargo/bin/k-shrink`):

```bash
mkdir -p ~/.config/systemd/user
# Download the service file from the repo contrib/ directory, then:
systemctl --user enable --now k-shrink.service
```

---

## Verifying

```bash
systemctl --user status k-shrink.service
# Copy an image, then paste — it should arrive compressed.
```

Logs:

```bash
journalctl --user -u k-shrink.service -f
```
