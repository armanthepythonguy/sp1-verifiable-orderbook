use models::state::State;
use rocket::{serde::json::Json};
use logic::state_logic::StateCol;

#[macro_use] extern crate rocket;
mod models;
mod logic;

#[get("/submit-state", format="json")]
fn get_state(state: &rocket::State<StateCol>) -> Option<Json<State>>{
    let res = state.get_state().unwrap();
    match res{
        Some(state) => {return Some(Json(state))},
        None => {return None}
    }
}

#[launch]
fn rocket() -> _ {
    let state = StateCol::init();
    rocket::build().manage(state)
        .mount("/", routes![get_state])
}