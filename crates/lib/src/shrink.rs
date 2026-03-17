use crate::{OutputFormat, ShrinkError};
use std::io::Cursor;

pub struct ShrinkOptions {
    pub quality: u8,
    pub target_format: OutputFormat,
}

#[derive(Debug)]
pub struct ShrinkResult {
    pub data: Vec<u8>,
    pub mime_type: String,
}

/// Decode arbitrary image bytes and re-encode as PNG.
/// Use this to produce a universally-pasteable fallback alongside a compressed format.
pub fn to_png(data: &[u8]) -> Result<Vec<u8>, ShrinkError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ShrinkError::DecodingFailed(e.to_string()))?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| ShrinkError::EncodingFailed(e.to_string()))?;
    Ok(buf)
}

pub fn shrink(data: &[u8], opts: &ShrinkOptions) -> Result<ShrinkResult, ShrinkError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ShrinkError::DecodingFailed(e.to_string()))?;

    let mut output = Vec::new();

    macro_rules! write_to {
        ($fmt:expr, $mime:expr) => {{
            img.write_to(&mut Cursor::new(&mut output), $fmt)
                .map_err(|e| ShrinkError::EncodingFailed(e.to_string()))?;
            $mime
        }};
    }

    let mime_type = match opts.target_format {
        // Lossy — quality knob applies
        OutputFormat::Jpeg => {
            img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut output,
                opts.quality,
            ))
            .map_err(|e| ShrinkError::EncodingFailed(e.to_string()))?;
            "image/jpeg"
        }
        OutputFormat::Avif => {
            img.write_with_encoder(image::codecs::avif::AvifEncoder::new_with_speed_quality(
                &mut output,
                6,
                opts.quality,
            ))
            .map_err(|e| ShrinkError::EncodingFailed(e.to_string()))?;
            "image/avif"
        }
        // Lossless — quality ignored
        OutputFormat::Webp    => write_to!(image::ImageFormat::WebP,     "image/webp"),
        OutputFormat::Png     => write_to!(image::ImageFormat::Png,      "image/png"),
        OutputFormat::Qoi     => write_to!(image::ImageFormat::Qoi,      "image/x-qoi"),
        OutputFormat::Farbfeld=> write_to!(image::ImageFormat::Farbfeld, "image/x-farbfeld"),
        OutputFormat::Tiff    => write_to!(image::ImageFormat::Tiff,     "image/tiff"),
        OutputFormat::Gif     => write_to!(image::ImageFormat::Gif,      "image/gif"),
        OutputFormat::Hdr     => write_to!(image::ImageFormat::Hdr,      "image/vnd.radiance"),
        OutputFormat::OpenExr => write_to!(image::ImageFormat::OpenExr,  "image/x-exr"),
        // Uncompressed (larger than source, but supported)
        OutputFormat::Bmp     => write_to!(image::ImageFormat::Bmp,      "image/bmp"),
        OutputFormat::Tga     => write_to!(image::ImageFormat::Tga,      "image/x-tga"),
        OutputFormat::Pnm     => write_to!(image::ImageFormat::Pnm,      "image/x-portable-anymap"),
        OutputFormat::Ico     => write_to!(image::ImageFormat::Ico,      "image/x-icon"),
    };

    Ok(ShrinkResult {
        data: output,
        mime_type: mime_type.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png() -> Vec<u8> {
        let img = image::DynamicImage::new_rgb8(8, 8);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    fn make_jpeg() -> Vec<u8> {
        let img = image::DynamicImage::new_rgb8(8, 8);
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    #[test]
    fn png_to_webp() {
        let result = shrink(
            &make_png(),
            &ShrinkOptions {
                quality: 80,
                target_format: OutputFormat::Webp,
            },
        )
        .unwrap();
        assert_eq!(result.mime_type, "image/webp");
        assert!(!result.data.is_empty());
    }

    #[test]
    fn jpeg_quality_50_produces_output() {
        let jpeg = make_jpeg();
        let result = shrink(
            &jpeg,
            &ShrinkOptions {
                quality: 50,
                target_format: OutputFormat::Jpeg,
            },
        )
        .unwrap();
        assert_eq!(result.mime_type, "image/jpeg");
        assert!(!result.data.is_empty());
    }

    #[test]
    fn random_bytes_decoding_failed() {
        let err = shrink(
            b"not an image at all",
            &ShrinkOptions {
                quality: 80,
                target_format: OutputFormat::Webp,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ShrinkError::DecodingFailed(_)));
    }

    #[test]
    fn quality_zero_produces_valid_output() {
        let result = shrink(
            &make_jpeg(),
            &ShrinkOptions {
                quality: 0,
                target_format: OutputFormat::Jpeg,
            },
        )
        .unwrap();
        assert!(!result.data.is_empty());
    }

    #[test]
    fn quality_max_produces_valid_output() {
        let result = shrink(
            &make_jpeg(),
            &ShrinkOptions {
                quality: 100,
                target_format: OutputFormat::Jpeg,
            },
        )
        .unwrap();
        assert!(!result.data.is_empty());
    }
}
