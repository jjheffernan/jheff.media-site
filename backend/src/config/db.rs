use mongodb::{options::ClientOptions, Client};
use std::{env, time::Duration};

pub async fn config_db() -> Client {
    let connection_string =
        env::var("YEW_FULLSTACK_DB_CONNSTR").unwrap_or(String::from("mongodb://localhost:27017"));
    let mut client_options = ClientOptions::parse(connection_string.as_str())
        .await
        .expect("Failed to parse MongoDB connection string");
    // Keep first-request hangs short when Mongo is waking up or unreachable.
    client_options.server_selection_timeout = Some(Duration::from_secs(5));
    client_options.connect_timeout = Some(Duration::from_secs(5));
    Client::with_options(client_options).expect("Failed to create MongoDB client")
}
