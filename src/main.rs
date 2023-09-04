use tokio_tungstenite::connect_async;
use futures_util::stream::StreamExt;
use std::env;

mod bot;
mod candlestick;
mod websocket;
mod message;
mod trend;
mod trade;

use bot::Bot;
use websocket::WebSocket;

#[tokio::main]
async fn main() {
    let url = env::var("WEBSOCKET_URL").expect("WEBSOCKET_URL must be set");

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("Connected to the server");

    let (write, read) = ws_stream.split();

    let ws = WebSocket::new(write);
    let bot = Bot::new(ws);

    bot.run(read).await;
}
