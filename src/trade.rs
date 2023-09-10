use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub direction: String,
    pub price: f64,
}

impl Trade {
    pub fn call(price: f64) -> Self {
        Self {
            direction: "call".to_string(),
            price,
        }
    }

    pub fn put(price: f64) -> Self {
        Self {
            direction: "put".to_string(),
            price,
        }
    }

    pub fn to_json(&self) -> String {
        json!({
            "action": "trade",
            "direction": self.direction,
            "price": self.price,
        })
        .to_string()
    }
}
