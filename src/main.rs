#![feature(proc_macro_hygiene, decl_macro)]
use rocket::Rocket;
use rocket_contrib::json::Json;
use chrono::prelude::*;
use api::course::{StudyLevel, Course};

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate serde_derive;

mod api;

const VERSION: &str = "v1";

fn rocket() -> Rocket {
    rocket::ignite()
        .mount(
            &format!("/api/{}/", VERSION),
            routes![course, pretty_course]
        )
}

fn run_course(code: String, year: Option<u32>, level: Option<String>) -> Option<Course> {
    let year = year
        .unwrap_or_else(|| chrono::Local::now().year() as u32);

    let level = level
        .map(|level| StudyLevel::from(&*level))
        .unwrap_or(StudyLevel::Undergrad);

    api::course::get_course(&*code, year, level)
}

#[get("/course/<code>?<year>&<level>")]
fn course(code: String, year: Option<u32>, level: Option<String>) -> Option<Json<Course>> {    
    run_course(code, year, level)
        .map(|course| Json(course))
}

#[get("/pretty/course/<code>?<year>&<level>")]
fn pretty_course(code: String, year: Option<u32>, level: Option<String>) -> Option<String> {
    run_course(code, year, level)
        .as_ref()
        .map(|course| serde_json::to_string_pretty(course).unwrap())
}

fn main() {
    rocket().launch();
}
