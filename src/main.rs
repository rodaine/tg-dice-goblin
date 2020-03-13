extern crate tokio;
extern crate futures;
extern crate telegram_bot;
extern crate rand;

mod parser;
mod handler;
mod roll;

use std::env;
use futures::StreamExt;
use telegram_bot::*;

const BOT_NAME: &str = "DiceGoblinBot";
const TOKEN_VAR: &str = "DICE_GOBLIN_TOKEN";

#[tokio::main]
async fn main() {
    let token = env::var(TOKEN_VAR)
        .unwrap_or_else(|_| panic!("{} not set", TOKEN_VAR));
    let api = Api::new(token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        if let Err(e) = update {
            eprintln!("Error from the stream: {}", e);
            continue;
        }

        if let Err(e) = handler::handle(&api, update.unwrap()).await {
            eprintln!("Error sending request: {}", e);
        }
    }
}
