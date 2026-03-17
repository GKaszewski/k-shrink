use lib::{image_hash, shrink, ClipboardError, ClipboardProvider, ShrinkOptions};
use tracing::{debug, info, trace};

pub fn process_once(
    provider: &dyn ClipboardProvider,
    opts: &ShrinkOptions,
    extra_mimes: &[String],
    last_hash: &mut Option<[u8; 32]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (data, mime_type) = match provider.capture() {
        Ok(x) => x,
        Err(ClipboardError::Empty) => {
            trace!("clipboard empty");
            return Ok(());
        }
        Err(ClipboardError::InvalidType(t)) => {
            trace!("no image in clipboard: {t}");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if !mime_type.starts_with("image/") {
        trace!("non-image mime type: {mime_type}");
        return Ok(());
    }

    let incoming_hash = image_hash(&data);
    if Some(incoming_hash) == *last_hash {
        debug!("clipboard unchanged, skipping");
        return Ok(());
    }

    info!("compressing {} bytes ({mime_type})", data.len());
    let result = shrink(&data, opts)?;
    info!(
        "compressed to {} bytes ({})",
        result.data.len(),
        result.mime_type
    );

    // Primary MIME type first, then any aliases the user wants to lie about.
    let mut items: Vec<(&[u8], &str)> = vec![(&result.data, result.mime_type.as_str())];
    for alias in extra_mimes {
        if alias != &result.mime_type {
            items.push((&result.data, alias.as_str()));
        }
    }
    provider.distribute(&items)?;

    *last_hash = Some(image_hash(&result.data));
    Ok(())
}
