# k-shrink

![AUR version](https://img.shields.io/aur/version/k-shrink)
![License](https://img.shields.io/badge/license-MIT-blue)

Wayland clipboard daemon that automatically compresses images the moment you copy them.
Copy a 3 MB screenshot, paste a 300 KB WebP — no extra steps.

## Quick Install (Arch Linux)

```bash
yay -S k-shrink
systemctl --user enable --now k-shrink.service
```

For other installation methods see [INSTALL.md](INSTALL.md).

## How It Works

k-shrink runs silently in the background. When it detects an image on the clipboard
(from a browser, image viewer, or file manager), it compresses it with your configured
format and quality and writes the result back. Your next paste delivers the smaller image.

A SHA-256 hash of the output bytes prevents k-shrink from reprocessing its own output,
so there are no infinite loops.

## Why?

Recently, I found myself sharing tons of memes and screenshots with friends, and it has come to my attention that many of those images are HUGE and it makes my blood boil. So, I decided to build a tool that will automatically take the image I have copied, whether from the web/facebook/discord/whatever or local files, or ftp, and shrink using user provided configuration (which format, quality, etc) and then put the shrunk image back to the clipboard, so that when I paste it, it's already shrunk and ready to be shared.

## Configuration

Config file: `~/.config/k-shrink/config.toml` (created with defaults if absent).

```toml
[general]
format    = "webp"   # output format
quality   = 80       # 0-100, only for jpeg/avif
poll_ms   = 500      # clipboard polling interval (ms, min 100)
extra_mimes = []     # additional MIME aliases (see below)
```

### Config Reference

| Field         | Type          | Default  | Description                                                                                                                                           |
| ------------- | ------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `format`      | string        | `"webp"` | Output image format. See formats table below.                                                                                                         |
| `quality`     | integer 0–100 | `80`     | Compression quality. Applied only for `jpeg` and `avif`. Ignored for all lossless formats.                                                            |
| `poll_ms`     | integer ≥ 100 | `500`    | How often to poll the clipboard (ms). Lower = more responsive, more CPU.                                                                              |
| `extra_mimes` | string array  | `[]`     | Extra MIME types to serve alongside the real one. Same compressed bytes, different label. Useful when the paste target only requests a specific type. |

### Supported Formats

| Format     | Lossy | Notes                                                   |
| ---------- | ----- | ------------------------------------------------------- |
| `webp`     | No    | Best for screenshots/UI. Smaller than PNG. **Default.** |
| `jpeg`     | Yes   | Best for photos. `quality` applies.                     |
| `avif`     | Yes   | Modern, better than JPEG. `quality` applies.            |
| `png`      | No    | Universal lossless. No size reduction vs source PNG.    |
| `qoi`      | No    | Fast lossless. Usually larger than PNG.                 |
| `farbfeld` | No    | Simple 16-bit lossless. Rarely needed.                  |
| `tiff`     | No    | Lossless TIFF. Professional workflows.                  |
| `gif`      | No    | 256 colors only. Avoid for photos.                      |
| `hdr`      | No    | Radiance HDR floating-point.                            |
| `openexr`  | No    | OpenEXR high dynamic range.                             |
| `bmp`      | No    | Uncompressed. Will be larger than source.               |
| `tga`      | No    | Uncompressed.                                           |
| `pnm`      | No    | Uncompressed.                                           |
| `ico`      | No    | Uncompressed.                                           |

## Troubleshooting

### Paste fails in Discord / Slack / browser

Some apps only request `image/png` even if other formats are available.
Use `extra_mimes` to serve the compressed bytes under that label:

```toml
[general]
format = "webp"
extra_mimes = ["image/png"]
```

The image is still encoded as WebP; the label is just what k-shrink advertises.
Most apps decode by content rather than MIME type, so this works.

### Images keep getting reprocessed (loop)

k-shrink uses a SHA-256 hash of its output to skip reprocessing. If you see a loop,
check that `IMAGE_MIMES` priority is intact (webp is listed first so k-shrink's own
output is matched before external types). This should not occur in normal usage.

### File-manager copies not detected

Filesystem copies (Ctrl+C in Nautilus, Dolphin, etc.) are detected via `text/uri-list`.
Only single-image files are supported. Multi-file selections and non-image files are
ignored silently.

## Portability

Wayland only. The architecture isolates platform code in the `platform` crate, so
X11/Windows/macOS backends can be added without touching business logic.

## Architecture

```
bin (k-shrink) → lib + platform + config
platform       → lib + wl-clipboard-rs
config         → lib + serde/toml/dirs
lib            → image + sha2  (zero platform deps)
```

- **lib** — pure image compression logic, no platform deps, fully unit-tested.
- **config** — TOML parsing, validation, and path resolution.
- **platform** — Wayland clipboard backend via `wl-clipboard-rs`.
- **bin** — tokio event loop; orchestrates the other three crates.

## Contributing

Contributions are welcome. Fork the repo and open a pull request. Please include
tests for new behavior and follow the existing code style.

## License

MIT. I seriously don't care what you do with this code or binaries, do whatever you want with it. If somehow it breaks something, it's your problem, not mine. #works-on-my-machine
