use futures_util::SinkExt;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::trade::Trade;

pub struct ExpertOption {
    write: futures_util::stream::SplitSink<
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
}

impl ExpertOption {
    pub fn new(
        write: futures_util::stream::SplitSink<
            WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message,
        >,
    ) -> Self {
        Self { write }
    }

    pub async fn send_trade(&mut self, trade: &Trade) {
        let msg = Message::Text(trade.to_json());
        self.write.send(msg).await.expect("Failed to send message");
    }
}
