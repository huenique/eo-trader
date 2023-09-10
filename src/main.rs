use futures_util::stream::StreamExt;
use std::env;
use tokio_tungstenite::connect_async;

mod bot;
mod candlestick;
mod expertoption;
mod message;
mod trade;
pub mod tradingview;
mod trend;

use bot::Bot;
use expertoption::ExpertOption;

#[tokio::main]
async fn main() {
    let url = env::var("EO_WEBSOCKET_URL").expect("EO_WEBSOCKET_URL must be set");

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("Connected to the server");

    let (write, read) = ws_stream.split();

    let ws = ExpertOption::new(write);
    let mut bot = Bot::new(ws);

    bot.run(read).await;
}
