# K-Shrink

Utility tool for shrinking/compressing images from the clipboard seamlessly. Built in Rust.

# Why?

Recently, I found myself sharing tons of memes and screenshots with friends, and it has come to my attention that many of those images are HUGE and it makes my blood boil. So, I decided to build a tool that will automatically take the image I have copied, whether from the web/facebook/discord/whatever or local files, or ftp, and shrink using user provided configuration (which format, quality, etc) and then put the shrunk image back to the clipboard, so that when I paste it, it's already shrunk and ready to be shared.

# How does it work?

It is basically a daemon that runs in the background and listens for clipboard changes. When it detects a change, it checks if the clipboard contains an image. If it does, it processes the image according to the user configuration and then puts the shrunk image back to the clipboard. We need some sort of lock or signature to prevent infinite loops.

# Configuration

The configuration is stored in a file called `config.toml` in ~/.config/k-shrink/ directory. The configuration file is in TOML format and contains the following fields:

```toml
[general]
# The format to shrink the image to. Supported formats are: png, jpeg, webp
format = "webp"
# The quality of the shrunk image. Supported values are: 0-100
quality = 80
```

# Portability

I personally use Arch Linux with Wayland, which is why for now it supports only wayland. But because I value portability and future-proofing, I have designed architecture in a way that it should be easy to add support for x11, windows, macos, what-have-you in the future.

# Architecture

That being said, let's talk about crates and general architecture.

All crates live in the `crates` directory. There are three main crates: `lib`, `platform`, and `bin`.

- `lib` crate is pure business logic, it doesn't care or know about the platform, it just provides the functionality to shrink images. It has no dependencies on platform-specific libraries, and it is tested with unit tests and integration tests. It is also the most stable crate, and it should not be changed frequently.
- `platform` crate is the crate that provides the platform-specific implementation of the clipboard provider. It depends on the `lib` crate, and it is tested with integration tests. It is also the most volatile crate, and it may change frequently as we add support for more platforms.
- `bin` crate is the crate that provides the binary executable. It depends on both `lib` and `platform` crates, and it is tested with integration tests. It is just an orchestrator that ties everything together, and it should not contain any business logic.

Configuration should be handled by new crate called `config`, which will take care of toml, reading and writing config. Also in `platform` crate we should provide a way for `config` crate to properly work across platforms. (this crate could be debated whether it should be separate or not, I am not convinced that separating it is the best idea, but we will see)

# Critical things I care about

- Performance: Entire point of this tool is to be invisible to the user, which means it's gotta be fast. it can't get in the way because then it defeats the purpose.
- Reliability: It should work consistently and not crash or cause any issues with the clipboard. It should also handle edge cases gracefully, such as unsupported formats, large images, etc.
- Portability: It should work on multiple platforms, and it should be easy to add support for new platforms in the future.
- User experience: It should be easy to use and configure, and it should provide feedback to the user when something goes wrong (e.g. unsupported format, etc). It should also have a good default configuration that works well for most users.
- Feature flags: We should use feature flags to enable or disable certain features, such as support for specific platforms, or support for specific image formats. This will allow us to keep the binary size small and avoid unnecessary dependencies for users who don't need those features.

# Contributing

Contributions are welcome! If you want to contribute, please fork the repository and create a pull request. Please make sure to follow the coding style and conventions used in the project. Also, please make sure to write tests for your changes.

# License

MIT. I seriously don't care what you do with this code or binaries, do whatever you want with it. If somehow it breaks something, it's your problem, not mine. #works-on-my-machine
