use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DollarQuote {
    pub id: i64,
    pub official_buy: f64,
    pub official_sell: f64,
    pub blue_buy: f64,
    pub blue_sell: f64,
    pub timestamp: String,
}

impl DollarQuote {
    pub fn new(official_buy: f64, official_sell: f64, blue_buy: f64, blue_sell: f64) -> Self {
        Self {
            id: 0,
            official_buy,
            official_sell,
            blue_buy,
            blue_sell,
            timestamp: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fields_round_trip_through_json() {
        let quote = DollarQuote::new(1000.5, 1040.75, 1200.0, 1240.5);

        let json = serde_json::to_string(&quote).unwrap();
        let back: DollarQuote = serde_json::from_str(&json).unwrap();

        assert_eq!(back, quote);
    }
}
