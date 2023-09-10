use std::ops::ControlFlow;

use rand::Rng;
use regex::Regex;
use reqwest;
use serde_json::{json, Value};
use websocket::header::{Headers, Origin};
use websocket::sync::Client;
use websocket::ClientBuilder;

/// Represents a WebSocket connection to TradingView.
pub struct TradingViewWebSocket {
    client: Client<std::net::TcpStream>,
}

impl TradingViewWebSocket {
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
        let message = self.create_message(func, args);
        self.client
            .send_message(&websocket::message::OwnedMessage::Text(message))
            .unwrap();
    }

    /// Creates a full message with header.
    fn create_message(&self, func: &str, param_list: Vec<Value>) -> String {
        let content = self.construct_message(func, param_list);
        self.prepend_header(&content)
    }

    /// Constructs a JSON message.
    fn construct_message(&self, func: &str, param_list: Vec<Value>) -> String {
        let message = json!({"m": func, "p": param_list});
        message.to_string()
    }

    /// Prepends header to content.
    fn prepend_header(&self, content: &str) -> String {
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
        println!("{}", symbol_id);
        symbol_id
    }

    /// Search for a symbol based on query and category.
    fn search(query: &str, category: &str) -> Result<Value, reqwest::Error> {
        let url = format!(
            "https://symbol-search.tradingview.com/symbol_search/?text={}&type={}",
            query, category
        );
        let response = reqwest::blocking::get(&url)?;
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
        result: &String,
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

            print!(
                "{}, price={}, change={}, change_percentage={}, volume={}\n",
                symbol,
                price.unwrap_or(0.0),
                change.unwrap_or(0.0),
                change_percentage.unwrap_or(0.0),
                volume.unwrap_or(0.0)
            );
        }
    }
}
