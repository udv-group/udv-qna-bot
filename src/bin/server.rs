use udv_qna_bot::{server::app::run_server, telemetry::init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    run_server().await;
    Ok(())
}
