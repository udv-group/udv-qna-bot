use cms::rocket;

#[rocket::main]
async fn main() {
    db::run_migrations().await.expect("Migrations failed");
    if let Err(e) = rocket().await.launch().await {
        drop(e);
    }
}
