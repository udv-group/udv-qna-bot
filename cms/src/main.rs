use cms::rocket;

#[rocket::main]
async fn main() {
    if let Err(e) = rocket().await.launch().await {
        drop(e);
    }
}
