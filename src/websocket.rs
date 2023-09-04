use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::trade::Trade;

type Write = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    Message,
>;

pub struct WebSocket {
    write: Write,
}

impl WebSocket {
    pub fn new(write: Write) -> Self {
        Self { write }
    }

    pub async fn send_trade(&mut self, trade: &Trade) {
        let msg = Message::Text(trade.to_json());
        self.write.send(msg).await.expect("Failed to send message");
    }
}
