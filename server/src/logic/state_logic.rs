use mongodb::sync::{Collection, Client};
use crate::models::state::State;

use std::env;
extern crate dotenv;
use dotenv::dotenv;

pub struct StateCol {
    col: Collection<State>
}

impl StateCol{

    pub fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGOURI"){
            Ok(v) => v.to_string(),
            Err(_) => format!("Error with env variable")
        };

        let client = Client::with_uri_str(uri).unwrap();
        let db = client.database("orderbook");
        let col: Collection<State> = db.collection("state");
        StateCol { col }
    }

    pub fn get_state(&self) -> Result<Option<State>, mongodb::error::Error> {
        self.col.find_one(None, None)
    }

}