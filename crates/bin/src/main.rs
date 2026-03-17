use config::load_config;
use lib::ShrinkOptions;
use platform::WaylandBackend;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = match load_config() {
        Ok(c) => c,
        Err(e) => {
            warn!("config error ({e}), using defaults");
            config::Config::default()
        }
    };

    let poll = Duration::from_millis(cfg.general.poll_ms);
    let extra_mimes = cfg.general.extra_mimes.clone();
    let opts = ShrinkOptions {
        quality: cfg.general.quality,
        target_format: cfg.general.format.into(),
    };

    let backend = WaylandBackend;
    let mut last_hash: Option<[u8; 32]> = None;

    info!("k-shrink daemon started (poll={}ms)", cfg.general.poll_ms);
    loop {
        if let Err(e) = k_shrink::process_once(&backend, &opts, &extra_mimes, &mut last_hash) {
            error!("error: {e}");
        }
        tokio::time::sleep(poll).await;
    }
}
