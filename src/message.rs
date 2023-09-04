use serde_json::Value;

pub enum Message {
    Candles(Vec<f64>),
    Unknown,
}

impl Message {
    pub fn from_text(text: &str) -> Self {
        let value: Value = serde_json::from_str(text).unwrap();

        match value["action"].as_str() {
            Some("candles") => {
                let candles = value["message"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
                Self::Candles(candles)
            }
            _ => Self::Unknown,
        }
    }
}
