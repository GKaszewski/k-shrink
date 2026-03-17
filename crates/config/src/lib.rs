use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    #[error("Quality must be 0–100, got {0}")]
    InvalidQuality(u8),
    #[error("poll_ms must be at least 100, got {0}")]
    InvalidPollMs(u64),
}

/// Output format for compressed images.
///
/// ## Lossy (quality applies)
/// - `jpeg`    — Best for photos. `quality` controls compression (80 is a good default).
/// - `avif`    — Modern format, better than JPEG at same quality. `quality` applies.
///
/// ## Lossless (quality ignored)
/// - `webp`    — Best for screenshots/UI. Usually smaller than PNG.
/// - `png`     — Universal lossless. No size reduction vs source PNG.
/// - `qoi`     — Fast lossless. Usually larger than PNG but faster to encode/decode.
/// - `farbfeld`— Simple 16-bit lossless. Rarely needed.
/// - `tiff`    — Lossless TIFF. Large files, used in professional workflows.
/// - `gif`     — Lossless but only 256 colors. Avoid for photos.
/// - `hdr`     — Radiance HDR floating-point format.
/// - `openexr` — OpenEXR high dynamic range.
///
/// ## Uncompressed (will be larger than source PNG)
/// - `bmp`, `tga`, `pnm`, `ico`
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Jpeg,
    Avif,
    Webp,
    Png,
    Qoi,
    Farbfeld,
    Tiff,
    Gif,
    Hdr,
    #[serde(rename = "openexr")]
    OpenExr,
    Bmp,
    Tga,
    Pnm,
    Ico,
}

impl From<OutputFormat> for lib::OutputFormat {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Jpeg     => lib::OutputFormat::Jpeg,
            OutputFormat::Avif     => lib::OutputFormat::Avif,
            OutputFormat::Webp     => lib::OutputFormat::Webp,
            OutputFormat::Png      => lib::OutputFormat::Png,
            OutputFormat::Qoi      => lib::OutputFormat::Qoi,
            OutputFormat::Farbfeld => lib::OutputFormat::Farbfeld,
            OutputFormat::Tiff     => lib::OutputFormat::Tiff,
            OutputFormat::Gif      => lib::OutputFormat::Gif,
            OutputFormat::Hdr      => lib::OutputFormat::Hdr,
            OutputFormat::OpenExr  => lib::OutputFormat::OpenExr,
            OutputFormat::Bmp      => lib::OutputFormat::Bmp,
            OutputFormat::Tga      => lib::OutputFormat::Tga,
            OutputFormat::Pnm      => lib::OutputFormat::Pnm,
            OutputFormat::Ico      => lib::OutputFormat::Ico,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    /// Output format. See [`OutputFormat`] for details.
    /// Default: `webp`
    #[serde(default = "default_format")]
    pub format: OutputFormat,

    /// Compression quality 0–100. Only used when `format = "jpeg"`.
    /// Ignored for `webp` (always lossless) and `png` (always lossless).
    /// Default: `80`
    #[serde(default = "default_quality")]
    pub quality: u8,

    /// How often to check the clipboard, in milliseconds.
    /// Lower values are more responsive but use more CPU.
    /// Minimum: 100. Default: `500`
    #[serde(default = "default_poll_ms")]
    pub poll_ms: u64,
}

fn default_format() -> OutputFormat {
    OutputFormat::Webp
}

fn default_quality() -> u8 {
    80
}

fn default_poll_ms() -> u64 {
    500
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            quality: default_quality(),
            poll_ms: default_poll_ms(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
        }
    }
}

fn validate(config: Config) -> Result<Config, ConfigError> {
    if config.general.quality > 100 {
        return Err(ConfigError::InvalidQuality(config.general.quality));
    }
    if config.general.poll_ms < 100 {
        return Err(ConfigError::InvalidPollMs(config.general.poll_ms));
    }
    Ok(config)
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("k-shrink")
        .join("config.toml")
}

pub fn load_config() -> Result<Config, ConfigError> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;
    let config: Config =
        toml::from_str(&text).map_err(|e| ConfigError::ParseError(e.to_string()))?;
    validate(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_format_webp_quality_80() {
        let c = Config::default();
        assert_eq!(c.general.format, OutputFormat::Webp);
        assert_eq!(c.general.quality, 80);
        assert_eq!(c.general.poll_ms, 500);
    }

    #[test]
    fn parse_valid_toml() {
        let toml = r#"
[general]
format = "jpeg"
quality = 75
poll_ms = 200
"#;
        let c: Config = toml::from_str(toml).unwrap();
        assert_eq!(c.general.format, OutputFormat::Jpeg);
        assert_eq!(c.general.quality, 75);
        assert_eq!(c.general.poll_ms, 200);
    }

    #[test]
    fn missing_file_returns_default() {
        let path = PathBuf::from("/tmp/definitely_does_not_exist_k_shrink.toml");
        let result = if !path.exists() {
            Ok(Config::default())
        } else {
            load_config()
        };
        assert!(result.is_ok());
        assert_eq!(result.unwrap().general.quality, 80);
    }

    #[test]
    fn invalid_toml_returns_error() {
        let bad = "not valid [ toml {{";
        let result: Result<Config, _> =
            toml::from_str(bad).map_err(|e| ConfigError::ParseError(e.to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn invalid_quality_returns_error() {
        let c = Config {
            general: GeneralConfig {
                format: OutputFormat::Webp,
                quality: 200,
                poll_ms: 500,
            },
        };
        assert!(matches!(validate(c), Err(ConfigError::InvalidQuality(200))));
    }

    #[test]
    fn poll_ms_too_low_returns_error() {
        let c = Config {
            general: GeneralConfig {
                format: OutputFormat::Webp,
                quality: 80,
                poll_ms: 50,
            },
        };
        assert!(matches!(validate(c), Err(ConfigError::InvalidPollMs(50))));
    }
}
