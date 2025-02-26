//! My messages will go here. If any messages are missing or have changed, please submit a pull
//! request or create an issue.

use super::Symbol;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Messages returned by the stream, 
/// require that you subscribe to the correct feed first.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Message {
    AggTrade(AggTrade),
    PartialDepth(PartialDepth),
    BookTicker(BookTicker),
    SubscribeSuccess { result: Option<String>, id: u8 },
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The Aggregate Trade Streams push trade information that is aggregated for a single taker order.
/// Update Speed: Real-time
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize )]
pub struct AggTrade {

    #[serde(alias = "E")]
    pub event_time: u64,
    
    #[serde(alias = "a")]
    pub trade_id: u64,

    #[serde(alias = "s")]
    pub symbol: Symbol,

    #[serde(alias = "p")]
    pub price: Decimal,

    #[serde(alias = "q")]
    pub quantity: Decimal,

    #[serde(alias = "f")]
    pub first_trade_id: u32,

    #[serde(alias = "l")]
    pub last_trade_id: u32,

    #[serde(alias = "T")]
    pub trade_time: u64,

    #[serde(alias = "m")]
    pub is_market_maker: bool,
}

/// Current Value of the Orderbook
/// Each level of Bids and Asks are Slices of length 2.
///
/// Containing [price, volume] as a [`Decimal`]
#[derive(Debug, Clone,  PartialEq, Eq, Serialize, Deserialize,)]
#[serde(rename_all = "camelCase")]
pub struct PartialDepth {
    pub last_update_id: u64,
    pub bids: Vec<[Decimal; 2]>,
    pub asks: Vec<[Decimal; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookTicker {
    #[serde(rename = "u")]
    update_id:u64,

    #[serde(rename = "s")]
    symbol:Symbol,

    // this can be reused in a BBO struct
    #[serde(rename = "b")]
    best_bid_price:Decimal,

    #[serde(rename = "B")]
    best_bid_qty: Decimal,

    #[serde(rename = "a")]
    best_ask_price:Decimal,

    #[serde(rename = "A")]
    best_ask_qty: Decimal
}

// TODO: Implement https://binance-docs.github.io/apidocs/spot/en/#all-market-mini-tickers-stream
// fun with nested BookTicker!


// Tests

#[cfg(test)]
const AGGTRADEMSG: &str = r#"
{
  "e":"aggTrade",
  "E":1591261134288,
  "a":424951,
  "s":"BTCUSDT",
  "p":"9643.5",
  "q":"2",
  "f":606073,
  "l":606073,
  "T":1591261134199,
  "m":false
}
"#;

#[cfg(test)]
const REALOB: &str = r#"{
"lastUpdateId":55130421061,
"bids":[
["98655.99000000","7.22497000"],
["98655.98000000","0.20352000"],
["98655.31000000","0.00100000"],
["98654.83000000","0.20251000"],
["98654.51000000","0.39110000"]],
"asks":[
["98656.00000000","0.00892000"],
["98656.01000000","0.00152000"],
["98656.02000000","0.00007000"],
["98656.04000000","0.00014000"],
["98659.98000000","0.00006000"]]}"#;


#[cfg(test)]
const BOOKTICKER: &str = r#"{
"u":400900217,
"s":"BNBUSDT",
"b":"25.35190000",
"B":"31.21000000",
"a":"25.36520000",
"A":"40.66000000"
}"#;

#[cfg(test)]
mod test {

    use super::*;
    use rust_decimal::{Decimal, prelude::FromPrimitive};

    #[test]
    fn book_ticker_parsing() {

        let parsed_bt: BookTicker = serde_json::from_str(BOOKTICKER).unwrap();

        let bt = BookTicker { 
            update_id: 400900217, 
            symbol: Symbol::BNBUSDT, 
            best_bid_price: Decimal::from_f64(25.35190000).unwrap(),
            best_bid_qty:   Decimal::from_f64(31.21000000).unwrap(),
            best_ask_price: Decimal::from_f64(25.36520000).unwrap(),
            best_ask_qty:   Decimal::from_f64(40.66000000).unwrap()
        };

        assert_eq!(bt, parsed_bt)
    }


    #[test]
    fn partial_ob_parsing() {

        let ob_msg: PartialDepth = serde_json::from_str(REALOB).unwrap();

        let depth = PartialDepth {
            last_update_id: 55130421061,
            bids: vec![
                [
                    Decimal::from_f64(98655.99000000).unwrap(),
                    Decimal::from_f64(7.22497000).unwrap(),
                ],
                [
                    Decimal::from_f64(98655.98000000).unwrap(),
                    Decimal::from_f64(0.20352000).unwrap(),
                ],
                [
                    Decimal::from_f64(98655.31000000).unwrap(),
                    Decimal::from_f64(0.00100000).unwrap(),
                ],
                [
                    Decimal::from_f64(98654.83000000).unwrap(),
                    Decimal::from_f64(0.20251000).unwrap(),
                ],
                [
                    Decimal::from_f64(98654.51000000).unwrap(),
                    Decimal::from_f64(0.39110000).unwrap(),
                ],
            ],
            asks: vec![
                [
                    Decimal::from_f64(98656.00000000).unwrap(),
                    Decimal::from_f64(0.00892000).unwrap(),
                ],
                [
                    Decimal::from_f64(98656.01000000).unwrap(),
                    Decimal::from_f64(0.00152000).unwrap(),
                ],
                [
                    Decimal::from_f64(98656.02000000).unwrap(),
                    Decimal::from_f64(0.00007000).unwrap(),
                ],
                [
                    Decimal::from_f64(98656.04000000).unwrap(),
                    Decimal::from_f64(0.00014000).unwrap(),
                ],
                [
                    Decimal::from_f64(98659.98000000).unwrap(),
                    Decimal::from_f64(0.00006000).unwrap(),
                ],
            ],
        };
        assert_eq!(depth, ob_msg)
    }

    #[test]
    fn partial_ob_binance_message() {
        let ob_msg: Message = serde_json::from_str(REALOB).unwrap();
        match ob_msg {
            Message::PartialDepth(_partial_depth) => assert_eq!(1, 1),
            _ => panic!("test failed"),
        };
    }

    #[test]
    fn aggtrade_message_parsing() {
        let t = AggTrade {
            event_time: 1591261134288,
            trade_id: 424951,
            symbol: Symbol::BTCUSDT,
            price: Decimal::from_f64(9643.5).unwrap(),
            quantity: Decimal::from_f32(2.0).unwrap(),
            first_trade_id: 606073,
            last_trade_id: 606073,
            trade_time: 1591261134199,
            is_market_maker: false,
        };
        let msg: AggTrade = serde_json::from_str(AGGTRADEMSG).unwrap();
        assert_eq!(t, msg)
    }

    #[test]
    fn api_message_aggtrade() {
        let t = AggTrade {
            event_time: 1591261134288,
            trade_id: 424951,
            symbol: Symbol::BTCUSDT,
            price: Decimal::from_f64(9643.5).unwrap(),
            quantity: Decimal::from_f32(2.0).unwrap(),
            first_trade_id: 606073,
            last_trade_id: 606073,
            trade_time: 1591261134199,
            is_market_maker: false,
        };
        let t = Message::AggTrade(t);

        let msg: Message = serde_json::from_str(AGGTRADEMSG).unwrap();

        assert_eq!(t, msg)
    }
}
