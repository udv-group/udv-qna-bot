mod categories;
mod error_handlers;
mod questions;
mod users;

use dotenv;
use rocket::Build;
use rocket_dyn_templates::Template;

#[macro_use]
extern crate rocket;

pub async fn rocket() -> rocket::Rocket<Build> {
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    rocket::build()
        .manage(db::establish_connection(&path).await.unwrap())
        .mount("/", users::routes())
        .mount("/", questions::routes())
        .mount("/", categories::routes())
        .attach(Template::fairing())
}
