use futures_util::stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;

use crate::websocket::WebSocket;
use crate::candlestick::Candlestick;
use crate::message::Message as BotMessage;
use crate::trend::Trend;
use crate::trade::Trade;

pub struct Bot {
    ws: WebSocket,
    trend: Trend,
    trades: Vec<Trade>,
}

impl Bot {
    pub fn new(ws: WebSocket) -> Self {
        Self {
            ws,
            trend: Trend::Unknown,
            trades: Vec::new(),
        }
    }

    pub async fn run(&mut self, mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin) {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let msg = BotMessage::from_text(&text);
                    self.handle_message(msg).await;
                }
                Ok(Message::Close(_)) => {
                    println!("Server closed the connection");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    async fn handle_message(&mut self, msg: BotMessage) {
        match msg {
            BotMessage::Candles(candles) => {
                let candlestick = Candlestick::from_candles(&candles);
                self.trend = candlestick.analyze_trend();
                self.execute_trades(&candlestick).await;
            }
            _ => {}
        }
    }

    async fn execute_trades(&mut self, candlestick: &Candlestick) {
        if self.trend == Trend::Up {
            if candlestick.has_long_tail() {
                let trade = Trade::call(candlestick.close);
                self.trades.push(trade);
                self.ws.send_trade(&trade).await;
            }
        } else if self.trend == Trend::Down {
            if candlestick.has_long_head() {
                let trade = Trade::put(candlestick.close);
                self.trades.push(trade);
                self.ws.send_trade(&trade).await;
            }
        }
    }
}
