use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> anyhow::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer().with_ansi(false);
    tracing_subscriber::registry().with(fmt_layer).init();
    Ok(())
}
