# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-17

### Added
- Initial public release.
- Wayland clipboard daemon: detects image copies and rewrites with compressed output.
- Supported output formats: `webp`, `jpeg`, `avif`, `png`, `qoi`, `farbfeld`, `tiff`,
  `gif`, `hdr`, `openexr`, `bmp`, `tga`, `pnm`, `ico`.
- `quality` option (0–100) for lossy formats (`jpeg`, `avif`).
- `poll_ms` option: configurable clipboard polling interval (min 100 ms).
- `extra_mimes` option: advertise compressed bytes under additional MIME aliases
  (useful for Discord, Slack, and browsers that request specific MIME types).
- Filesystem image detection via `text/uri-list` (file manager copies).
- SHA-256 deduplication to prevent infinite recompression loops.
- Workspace-based multi-crate architecture: `lib`, `config`, `platform`, `bin`.
- systemd user service (`contrib/k-shrink.service`).
- AUR PKGBUILD (`contrib/PKGBUILD`).
- Man page (`man/k-shrink.1`).
