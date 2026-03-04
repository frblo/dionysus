use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

pub fn init_tracing(config: &Config) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_new(config.logging.filter.clone())?;

    let fmt_layer = tracing_subscriber::fmt::layer();

    if config.logging.json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer.json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }

    Ok(())
}
