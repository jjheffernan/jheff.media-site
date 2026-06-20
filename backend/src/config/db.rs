use mongodb::{options::ClientOptions, Client};
use std::env;

pub async fn config_db() -> Client {
    let connection_string =
        env::var("YEW_FULLSTACK_DB_CONNSTR").unwrap_or(String::from("mongodb://localhost:27017"));
    let client_options = ClientOptions::parse(connection_string.as_str())
        .await
        .expect("Failed to parse MongoDB connection string");
    Client::with_options(client_options).expect("Failed to create MongoDB client")
}
