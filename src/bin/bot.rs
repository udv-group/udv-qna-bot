use udv_qna_bot::{bot::start_bot, telemetry::init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    start_bot().await
}
