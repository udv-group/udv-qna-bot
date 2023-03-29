mod categories;
mod error_handlers;
mod questions;
mod users;

use rocket::{response::Redirect, Build};
use rocket_dyn_templates::Template;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(questions::get_questions))
}
pub async fn rocket() -> rocket::Rocket<Build> {
    let path = dotenv::var("DB_PATH").expect("DB_PATH must be set");
    rocket::build()
        .manage(db::establish_connection(&path).await.unwrap())
        .mount("/", routes![index])
        .mount("/", users::routes())
        .mount("/", questions::routes())
        .mount("/", categories::routes())
        .attach(Template::fairing())
}
