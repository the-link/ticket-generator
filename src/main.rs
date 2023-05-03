#[macro_use]
extern crate rocket;

use our_application::catchers;
use our_application::fairings::{csrf::Csrf, db::DBConnection};
use our_application::routes::{self, issues_reported, user};
use rocket::fs::relative;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

#[launch]
async fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(DBConnection::init())
        .attach(Template::fairing())
        .attach(Csrf::new())
        .mount(
            "/",
            routes![
                routes::shutdown,
                issues_reported::get_issue,
                issues_reported::get_issues,
                issues_reported::manage_issues,
                issues_reported::new_issue,
                issues_reported::create_issue,
                issues_reported::edit_issue,
                issues_reported::update_issue,
                issues_reported::put_issue,
                issues_reported::patch_issue,
                issues_reported::delete_issue,
                issues_reported::delete_issue_entry_point,
                issues_reported::get_open,
                issues_reported::complete, 
                user::get_user,
                user::get_users,
                user::new_user,
                user::create_user,
                user::edit_user,
                user::update_user,
                user::put_user,
                user::patch_user,
                user::delete_user,
                user::delete_user_entry_point,     


            ],
        )
        .mount("/assets", FileServer::from(relative!("static")))
        .register(
            "/",
            catchers![
                catchers::bad_request,
                catchers::not_found,
                catchers::unprocessable_entity,
                catchers::internal_server_error
            ],
        )
}
