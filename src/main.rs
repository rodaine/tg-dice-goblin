use std::env;
use std::error::Error;
use std::pin::pin;

use grammers_client::{Client, Config, InitParams};
use grammers_session::Session;
use log::{error, trace};
use tokio::{select, task};

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod handler;
mod rolls;

const API_ID_VAR: &str = "DICE_GOBLIN_API_ID";
const API_HASH_VAR: &str = "DICE_GOBLIN_API_HASH";
const TOKEN_VAR: &str = "DICE_GOBLIN_TOKEN";
const SESSION_VAR: &str = "DICE_GOBLIN_SESSION";

type Result = std::result::Result<(), Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result {
    simple_logger::init_with_env()?;

    let session_file = env::var(SESSION_VAR)?;
    let api_id = env::var(API_ID_VAR)?.parse()?;
    let api_hash = env::var(API_HASH_VAR)?;
    let token = env::var(TOKEN_VAR)?;

    trace!("connecting to Telegram...");
    let client = Client::connect(
        Config {
            api_id,
            api_hash,
            session: Session::load_file_or_create(&session_file)?,
            params: InitParams {
                catch_up: true,
                ..Default::default()
            },
        }
    ).await?;
    trace!("connected");

    if !client.is_authorized().await? {
        trace!("Signing in...");
        client.bot_sign_in(&token).await?;
        if let Err(e) =  client.session().save_to_file(&session_file) {
            client.sign_out().await?;
            return Err(e.into());
        }
        trace!("Signed in!")
    }

    let mut exit = pin!(tokio::signal::ctrl_c());
    loop {
        let update = select! {
            _ = &mut exit => None,
            upd = client.next_update() => upd?,
        };
        match update {
            None => break,
            Some(upd) => task::spawn(async move {
                match handler::handle(upd).await {
                    Ok(_) => {},
                    Err(e) => error!("Error handling update: {}", e)
                }
            }),
        };
    }

    trace!("Exiting...");
    client.session().save_to_file(&session_file)?;
    Ok(())
}