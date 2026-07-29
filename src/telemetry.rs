use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> anyhow::Result<()> {
    let filter = EnvFilter::new("INFO");
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(filter);
    tracing_subscriber::registry().with(fmt_layer).init();
    Ok(())
}
