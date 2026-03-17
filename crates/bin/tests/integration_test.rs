use lib::{ClipboardError, ClipboardProvider, OutputFormat, ShrinkOptions};
use std::cell::Cell;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

struct MockClipboard {
    capture_bytes: Vec<u8>,
    capture_mime: String,
    distributed: Arc<Mutex<Vec<(Vec<u8>, String)>>>,
    capture_count: Cell<usize>,
}

impl MockClipboard {
    fn new(bytes: Vec<u8>, mime: &str) -> Self {
        Self {
            capture_bytes: bytes,
            capture_mime: mime.to_string(),
            distributed: Arc::new(Mutex::new(Vec::new())),
            capture_count: Cell::new(0),
        }
    }
}

impl ClipboardProvider for MockClipboard {
    fn capture(&self) -> Result<(Vec<u8>, String), ClipboardError> {
        self.capture_count.set(self.capture_count.get() + 1);
        Ok((self.capture_bytes.clone(), self.capture_mime.clone()))
    }

    fn distribute(&self, items: &[(&[u8], &str)]) -> Result<(), ClipboardError> {
        let mut lock = self.distributed.lock().unwrap();
        for (data, mime) in items {
            lock.push((data.to_vec(), mime.to_string()));
        }
        Ok(())
    }
}

fn make_png() -> Vec<u8> {
    let img = image::DynamicImage::new_rgb8(8, 8);
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    buf
}

fn webp_opts() -> ShrinkOptions {
    ShrinkOptions {
        quality: 80,
        target_format: OutputFormat::Webp,
    }
}

#[test]
fn image_bytes_are_shrunk_and_distributed_as_webp() {
    use k_shrink::process_once;

    let png = make_png();
    let mock = MockClipboard::new(png, "image/png");
    let mut last_hash = None;

    process_once(&mock, &webp_opts(), &mut last_hash).unwrap();

    let dist = mock.distributed.lock().unwrap();
    assert_eq!(dist.len(), 1, "exactly one item distributed");
    assert_eq!(dist[0].1, "image/webp");
}

#[test]
fn same_webp_output_not_reprocessed() {
    use k_shrink::process_once;

    let png = make_png();
    let mock = MockClipboard::new(png, "image/png");
    let mut last_hash = None;

    process_once(&mock, &webp_opts(), &mut last_hash).unwrap();

    // After distributing, our subprocess serves image/webp.
    // Simulate next tick: clipboard returns the webp we just wrote.
    let webp_data = mock.distributed.lock().unwrap()[0].0.clone();
    let mock2 = MockClipboard::new(webp_data, "image/webp");

    process_once(&mock2, &webp_opts(), &mut last_hash).unwrap();

    // hash(webp) == last_hash → skipped
    assert_eq!(mock2.distributed.lock().unwrap().len(), 0);
}

#[test]
fn non_image_mime_not_processed() {
    use k_shrink::process_once;

    let mock = MockClipboard::new(b"hello world".to_vec(), "text/plain");
    let mut last_hash = None;

    process_once(&mock, &webp_opts(), &mut last_hash).unwrap();

    assert_eq!(mock.distributed.lock().unwrap().len(), 0);
}
