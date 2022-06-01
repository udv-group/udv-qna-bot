mod categories;
mod error_handlers;
mod questions;
mod users;

use rocket::Build;

#[macro_use]
extern crate rocket;

pub async fn rocket() -> rocket::Rocket<Build> {
    rocket::build()
        .manage(db::establish_connection().await.unwrap())
        .mount("/", users::routes())
        .mount("/", questions::routes())
        .mount("/", categories::routes())
}
