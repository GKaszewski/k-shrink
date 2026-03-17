use lib::{ClipboardError, ClipboardProvider};
use std::io::Read;
use tracing::debug;

pub struct WaylandBackend;

const IMAGE_MIMES: &[&str] = &[
    // Compressed — try our own likely output first
    "image/webp",
    "image/avif",
    // Common lossless/lossy
    "image/png",
    "image/jpeg",
    // Other standard types
    "image/gif",
    "image/bmp",
    "image/tiff",
    // Less common but image-crate-supported
    "image/x-qoi",
    "image/x-tga",
    "image/x-icon",
    "image/vnd.radiance",
    "image/x-exr",
    "image/x-portable-anymap",
    "image/x-farbfeld",
];

impl ClipboardProvider for WaylandBackend {
    fn capture(&self) -> Result<(Vec<u8>, String), ClipboardError> {
        use wl_clipboard_rs::paste::{ClipboardType, Error, MimeType, Seat, get_contents};

        // 1. Try raw image MIME types (webp first so our own output is matched first).
        for &mime in IMAGE_MIMES {
            match get_contents(
                ClipboardType::Regular,
                Seat::Unspecified,
                MimeType::Specific(mime),
            ) {
                Ok((mut pipe, mime_type)) => {
                    let mut data = Vec::new();
                    pipe.read_to_end(&mut data)
                        .map_err(|e| ClipboardError::BackendError(e.to_string()))?;
                    if data.is_empty() {
                        debug!("{mime} offered but data was empty");
                        continue;
                    }
                    debug!("captured {} bytes as {mime_type}", data.len());
                    return Ok((data, mime_type));
                }
                Err(Error::NoMimeType) => {
                    debug!("{mime} not offered");
                    continue;
                }
                Err(Error::NoSeats) | Err(Error::ClipboardEmpty) => {
                    return Err(ClipboardError::Empty);
                }
                Err(e) => return Err(ClipboardError::BackendError(e.to_string())),
            }
        }

        // 2. Fallback: file manager copies put file URIs in text/uri-list.
        match get_contents(
            ClipboardType::Regular,
            Seat::Unspecified,
            MimeType::Specific("text/uri-list"),
        ) {
            Ok((pipe, _)) => read_image_from_uri_list(pipe),
            Err(Error::NoMimeType) | Err(Error::NoSeats) | Err(Error::ClipboardEmpty) => {
                Err(ClipboardError::Empty)
            }
            Err(e) => Err(ClipboardError::BackendError(e.to_string())),
        }
    }

    fn distribute(&self, items: &[(&[u8], &str)]) -> Result<(), ClipboardError> {
        use wl_clipboard_rs::copy::{MimeSource, MimeType as CopyMime, Options, Source};

        let sources: Vec<MimeSource> = items
            .iter()
            .map(|(data, mime)| MimeSource {
                source: Source::Bytes(data.to_vec().into_boxed_slice()),
                mime_type: CopyMime::Specific(mime.to_string()),
            })
            .collect();

        Options::new()
            .copy_multi(sources)
            .map_err(|e| ClipboardError::BackendError(e.to_string()))
    }
}

fn read_image_from_uri_list(mut pipe: impl Read) -> Result<(Vec<u8>, String), ClipboardError> {
    let mut content = String::new();
    pipe.read_to_string(&mut content)
        .map_err(|e| ClipboardError::BackendError(e.to_string()))?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let path_str = match line.strip_prefix("file://") {
            Some(p) => percent_decode(p),
            None => {
                debug!("skipping non-file URI: {line}");
                continue;
            }
        };

        let path = std::path::Path::new(&path_str);
        let mime = match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref()
        {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("webp") => "image/webp",
            Some("avif") => "image/avif",
            Some("gif") => "image/gif",
            Some("bmp") => "image/bmp",
            Some("tiff") | Some("tif") => "image/tiff",
            Some("qoi") => "image/x-qoi",
            Some("tga") => "image/x-tga",
            Some("ico") => "image/x-icon",
            Some("hdr") => "image/vnd.radiance",
            Some("exr") => "image/x-exr",
            Some("ppm") | Some("pgm") | Some("pbm") | Some("pam") => "image/x-portable-anymap",
            Some("ff") => "image/x-farbfeld",
            _ => {
                debug!("skipping non-image file: {path_str}");
                continue;
            }
        };

        debug!("reading image file: {path_str}");
        let data = std::fs::read(path)
            .map_err(|e| ClipboardError::BackendError(format!("{path_str}: {e}")))?;

        return Ok((data, mime.to_string()));
    }

    Err(ClipboardError::InvalidType(
        "no image files in uri-list".to_string(),
    ))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_spaces() {
        assert_eq!(
            percent_decode("/home/user/my%20file.png"),
            "/home/user/my file.png"
        );
    }

    #[test]
    fn percent_decode_no_encoding() {
        assert_eq!(
            percent_decode("/home/user/photo.jpg"),
            "/home/user/photo.jpg"
        );
    }

    #[test]
    fn uri_list_skips_comments_and_non_image() {
        let input = b"# comment\nfile:///home/user/doc.pdf\nfile:///home/user/photo.png\n";
        let result = read_image_from_uri_list(input.as_slice());
        // doc.pdf skipped, photo.png read → would fail because file doesn't exist,
        // but we get a BackendError (not InvalidType), meaning we got past the extension check.
        assert!(matches!(result, Err(ClipboardError::BackendError(_))));
    }

    #[test]
    fn uri_list_no_images_returns_invalid_type() {
        let input = b"file:///home/user/doc.pdf\nfile:///home/user/notes.txt\n";
        let result = read_image_from_uri_list(input.as_slice());
        assert!(matches!(result, Err(ClipboardError::InvalidType(_))));
    }

    #[test]
    #[ignore]
    fn round_trip() {
        let backend = WaylandBackend;
        let data = b"fake image bytes";
        backend.distribute(&[(data, "image/webp")]).unwrap();
        let (got, mime) = backend.capture().unwrap();
        assert_eq!(got, data);
        assert_eq!(mime, "image/webp");
    }
}
