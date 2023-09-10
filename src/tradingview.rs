use std::ops::ControlFlow;

use rand::Rng;
use regex::Regex;
use reqwest;
use serde_json::{json, Value};
use websocket::header::{Headers, Origin};
use websocket::sync::Client;
use websocket::ClientBuilder;

/// Establishes a WebSocket connection to TradingView and starts a job to receive quotes for a given pair and market.
///
/// # Arguments
///
/// * `pair` - A string slice that holds the trading pair to receive quotes for.
/// * `market` - A string slice that holds the market to receive quotes for.
///
/// # Example
///
/// ```
/// tradingview_ws("btcusdt", "crypto");
/// ```
///
/// # Panics
///
/// The function panics if the WebSocket connection cannot be established.
pub fn tradingview_ws(pair: &str, market: &str) {
    let trading_view_socket = "wss://data.tradingview.com/socket.io/websocket";

    let mut headers = Headers::new();
    headers.set(Origin("https://data.tradingview.com".to_string()));

    let mut client = ClientBuilder::new(trading_view_socket)
        .unwrap()
        .custom_headers(&headers)
        .connect_insecure()
        .unwrap();

    let session = generate_session();

    send_message(
        &mut client,
        "quote_create_session",
        vec![json!(session.clone())],
    );
    send_message(
        &mut client,
        "quote_set_fields",
        vec![
            json!(session.clone()),
            serde_json::Value::String("lp".to_string()),
            serde_json::Value::String("volume".to_string()),
            serde_json::Value::String("ch".to_string()),
            serde_json::Value::String("chp".to_string()),
        ],
    );
    send_message(
        &mut client,
        "quote_add_symbols",
        vec![json!(session), json!(get_symbol_id(pair, market))],
    );

    socket_job(&mut client);
}

/// Send a message over the WebSocket connection
///
/// # Arguments
///
/// * `ws` - A mutable reference to a `Client` object that holds the WebSocket connection.
/// * `func` - A string slice that holds the name of the function.
/// * `args` - A vector of `Value` objects that holds the parameters of the function.
pub fn send_message(ws: &mut Client<std::net::TcpStream>, func: &str, args: Vec<Value>) {
    let message = create_message(func, args);
    ws.send_message(&websocket::message::OwnedMessage::Text(message))
        .unwrap();
}

/// Create a full message with header
///
/// # Arguments
///
/// * `func` - A string slice that holds the name of the function.
/// * `param_list` - A vector of `Value` objects that holds the parameters of the function.
///
/// # Returns
///
/// A `String` containing the full message with a header.
fn create_message(func: &str, param_list: Vec<Value>) -> String {
    let content = construct_message(func, param_list);
    prepend_header(&content)
}

/// Construct a JSON message
///
/// # Arguments
///
/// * `func` - A string slice that holds the name of the function.
/// * `param_list` - A vector of `Value` objects that holds the parameters of the function.
///
/// # Returns
///
/// A `String` containing the constructed JSON message.
fn construct_message(func: &str, param_list: Vec<Value>) -> String {
    let message = json!({"m": func, "p": param_list});
    message.to_string()
}

/// Prepend header to content
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to be prepended with a header.
///
/// # Returns
///
/// A `String` containing the content with a prepended header.
fn prepend_header(content: &str) -> String {
    format!("~m~{}~m~{}", content.len(), content)
}

/// Get symbol ID based on pair and market
///
/// # Arguments
///
/// * `pair` - A string slice that holds the pair value
/// * `market` - A string slice that holds the market value
///
/// # Returns
///
/// A string that represents the symbol ID
pub fn get_symbol_id(pair: &str, market: &str) -> String {
    let data = search(pair, market).unwrap();
    let symbol_name = data["symbol"].as_str().unwrap();
    let broker = data
        .get("prefix")
        .and_then(|prefix| prefix.as_str())
        .unwrap_or_else(|| data["exchange"].as_str().unwrap());
    let symbol_id = format!("{}:{}", broker.to_uppercase(), symbol_name.to_uppercase());
    println!("{}", symbol_id);
    symbol_id
}

/// Search for a symbol based on query and category
///
/// # Arguments
///
/// * `query` - A string slice that holds the search query.
/// * `category` - A string slice that holds the category of the search.
///
/// # Returns
///
/// A `Result` containing a `Value` object if the search is successful, or a `reqwest::Error` if the search fails.
pub fn search(query: &str, category: &str) -> Result<Value, reqwest::Error> {
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

/// Generate a random session ID
///
/// # Returns
///
/// A `String` containing a random session ID.
pub fn generate_session() -> String {
    let string_length = 12;
    let letters = "abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let random_string: String = (0..string_length)
        .map(|_| rng.gen_range(0..letters.len()))
        .map(|i| letters.chars().nth(i).unwrap())
        .collect();
    format!("qs_{}", random_string)
}

/// Sends a ping packet to the WebSocket server.
///
/// # Arguments
///
/// * `ws` - A mutable reference to the WebSocket client.
/// * `result` - A string slice containing the result to parse.
///
/// # Example
///
/// ```
/// let mut ws = Client::<TcpStream>::connect(url).unwrap();
/// let result = ws.recv_message().unwrap().into_text().unwrap();
/// send_ping_packet(&mut ws, &result);
/// ```
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

/// Handles WebSocket messages received by the client.
///
/// # Arguments
///
/// * `ws` - A mutable reference to the WebSocket client.
pub fn socket_job(ws: &mut Client<std::net::TcpStream>) {
    let regex_ = Regex::new(r"^.*?({.*)}$").unwrap();

    loop {
        let result = ws.recv_message().unwrap();
        match result {
            websocket::message::OwnedMessage::Text(ref result) => {
                if let ControlFlow::Break(_) = get_price(result, &regex_, ws) {
                    continue;
                }
            }
            _ => {}
        }
    }
}

/// Parses the `result` string using the provided `regex_` and extracts the price, volume, change, and change percentage
/// information for a given symbol. If the `result` string contains "quote_completed" or "session_id", the function
/// returns `ControlFlow::Break(())` to stop the loop. Otherwise, the function prints the extracted information to the console
/// and returns `ControlFlow::Continue(())`.
///
/// # Arguments
///
/// * `result` - A reference to a `String` containing the result string to parse.
/// * `regex_` - A reference to a `Regex` object used to extract the JSON string from the `result` string.
/// * `ws` - A mutable reference to a `Client` object used to send a ping packet if the `result` string cannot be parsed.
///
/// # Returns
///
/// * `ControlFlow::Break(())` if the `result` string contains "quote_completed" or "session_id".
/// * `ControlFlow::Continue(())` otherwise.
fn get_price(
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
        parse_price_data(json_str);
    } else {
        send_ping_packet(ws, result);
    }

    ControlFlow::Continue(())
}

/// Parses the price data from a JSON string and prints the symbol, price, change, change percentage, and volume.
///
/// # Arguments
///
/// * `json_str` - A string slice that holds the JSON data.
///
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
