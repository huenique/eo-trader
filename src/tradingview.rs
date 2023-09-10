use std::ops::ControlFlow;

use rand::Rng;
use regex::Regex;
use reqwest;
use serde_json::{json, Value};
use websocket::header::{Headers, Origin};
use websocket::sync::Client;
use websocket::ClientBuilder;

/// Represents a WebSocket connection to TradingView.
pub struct TradingView {
    client: Client<std::net::TcpStream>,
}

impl Default for TradingView {
    fn default() -> Self {
        Self::new()
    }
}

impl TradingView {
    /// Creates a new WebSocket connection to TradingView.
    pub fn new() -> Self {
        let trading_view_socket = "wss://data.tradingview.com/socket.io/websocket";

        let mut headers = Headers::new();
        headers.set(Origin("https://data.tradingview.com".to_string()));

        let client = ClientBuilder::new(trading_view_socket)
            .unwrap()
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap();

        Self { client }
    }

    /// Sends a message over the WebSocket connection.
    pub fn send_message(&mut self, func: &str, args: Vec<Value>) {
        let message = Self::create_message(func, args);
        self.client
            .send_message(&websocket::message::OwnedMessage::Text(message))
            .unwrap();
    }

    /// Creates a full message with header.
    fn create_message(func: &str, param_list: Vec<Value>) -> String {
        let content = Self::construct_message(func, param_list);
        Self::prepend_header(&content)
    }

    /// Constructs a JSON message.
    fn construct_message(func: &str, param_list: Vec<Value>) -> String {
        let message = json!({"m": func, "p": param_list});
        message.to_string()
    }

    /// Prepends header to content.
    fn prepend_header(content: &str) -> String {
        format!("~m~{}~m~{}", content.len(), content)
    }
}

/// Represents a symbol in the TradingView system.
pub struct Symbol {
    pub symbol_id: String,
}

impl Symbol {
    /// Gets symbol ID based on pair and market.
    pub fn get_symbol_id(pair: &str, market: &str) -> String {
        let data = Self::search(pair, market).unwrap();
        let symbol_name = data["symbol"].as_str().unwrap();
        let broker = data
            .get("prefix")
            .and_then(|prefix| prefix.as_str())
            .unwrap_or_else(|| data["exchange"].as_str().unwrap());
        let symbol_id = format!("{}:{}", broker.to_uppercase(), symbol_name.to_uppercase());
        symbol_id
    }

    /// Search for a symbol based on query and category.
    fn search(query: &str, category: &str) -> Result<Value, reqwest::Error> {
        let url = format!(
            "https://symbol-search.tradingview.com/symbol_search/?text={}&type={}",
            query, category
        );
        let response = reqwest::blocking::get(url)?;
        if response.status().is_success() {
            let data: Value = response.json()?;
            assert!(!data.is_null(), "Nothing Found.");
            Ok(data[0].clone())
        } else {
            Err(response.error_for_status().unwrap_err())
        }
    }
}

/// Represents a TradingView session.
pub struct Session {
    pub session_id: String,
}

impl Session {
    /// Generates a random session ID.
    pub fn generate_session() -> Self {
        let string_length = 12;
        let letters = "abcdefghijklmnopqrstuvwxyz";
        let mut rng = rand::thread_rng();
        let random_string: String = (0..string_length)
            .map(|_| rng.gen_range(0..letters.len()))
            .map(|i| letters.chars().nth(i).unwrap())
            .collect();
        Session {
            session_id: format!("qs_{}", random_string),
        }
    }
}

/// Represents the WebSocket job.
pub struct SocketJob;

impl SocketJob {
    /// Handles WebSocket messages received by the client.
    pub fn handle_message(
        result: &str,
        regex_: &Regex,
        ws: &mut Client<std::net::TcpStream>,
    ) -> ControlFlow<()> {
        if result.contains("quote_completed") || result.contains("session_id") {
            return ControlFlow::Break(());
        }

        let res = regex_
            .captures(result)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()));

        if let Some(json_str) = res {
            Self::parse_price_data(json_str);
        } else {
            Self::send_ping_packet(ws, result);
        }

        ControlFlow::Continue(())
    }

    /// Sends a ping packet to the WebSocket server.
    fn send_ping_packet(ws: &mut Client<std::net::TcpStream>, result: &str) {
        let ping_str = Regex::new(r".......(.*)")
            .unwrap()
            .captures(result)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()))
            .unwrap_or("");

        if !ping_str.is_empty() {
            let ping_message = format!("~m~{}~m~{}", ping_str.len(), ping_str);
            ws.send_message(&websocket::message::OwnedMessage::Text(ping_message))
                .unwrap();
        }
    }

    /// Parses the price data from a JSON string and prints the symbol, price, change, change percentage, and volume.
    fn parse_price_data(json_str: &str) {
        let json_res: Value = serde_json::from_str(json_str).unwrap();
        if let Some(prefix) = json_res["p"][1].as_object() {
            let symbol = prefix["n"].as_str().unwrap();
            let price = prefix["v"]["lp"].as_f64();
            let volume = prefix["v"]["volume"].as_f64();
            let change = prefix["v"]["ch"].as_f64();
            let change_percentage = prefix["v"]["chp"].as_f64();

            println!(
                "{}, price={}, change={}, change_percentage={}, volume={}",
                symbol,
                price.unwrap_or(0.0),
                change.unwrap_or(0.0),
                change_percentage.unwrap_or(0.0),
                volume.unwrap_or(0.0)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_message() {
        let func = "test_func";
        let args = vec![json!("arg1"), json!(2)];
        let expected = "~m~32~m~{\"m\":\"test_func\",\"p\":[\"arg1\",2]}";
        let actual = TradingView::create_message(func, args);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_prepend_header() {
        let content = "Hello, world!";
        let expected_result = format!("~m~{}~m~{}", content.len(), content);
        let result = TradingView::prepend_header(content);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_get_symbol_id() {
        let expected = "BINANCE:BTCUSDT".to_string();
        let actual = Symbol::get_symbol_id("btcusdt", "crypto");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_generate_session() {
        let session = Session::generate_session();
        assert_eq!(session.session_id.len(), 15);
        assert!(session.session_id.starts_with("qs_"));
    }

    #[test]
    fn test_parse_price_data() {
        let json_str = "{\"m\":\"q\",\"p\":[{\"n\":\"BTCUSDT:CRYPTO\",\"v\":{\"lp\":34200.0,\"ch\":-0.0005,\"chp\":-0.0015,\"volume\":0.0}}]}";
        SocketJob::parse_price_data(json_str);
    }
}
