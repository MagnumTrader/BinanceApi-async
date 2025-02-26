//! BinanceApi-async provides a standardized way to stream data from Binance public Api.
//!
//! You will recieve the messages as standardized struct, see [`Message`]
//!
//! **Official docs:** https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams
pub mod messages;
pub use messages::Message;
mod symbol;
pub use symbol::{subscribe_msg_all_symbols, Symbol};
mod error;
pub use error::Error;

use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;
use tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};
use tracing::{error, info, warn};

type Result<T> = std::result::Result<T, crate::Error>;

const APIURL: &str = "wss://stream.binance.com:9443/ws";
// seems to be a URL for trading etc not data streaming
// const APIURL: &str = "wss://ws-api.binance.com:9443/ws-api/v3";

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct BinanceApi {
    stream: Option<WsStream>,
    connected: bool,
}

impl Default for BinanceApi {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceApi {
    /// Create a new instance of BinanceApi, not connected.
    /// Use [`BinanceApi::connect()`] to connect.
    pub fn new() -> Self {
        Self {
            stream: None,
            connected: false,
        }
    }

    /// Establishes a Websocket connection to Binance Public Api.
    ///
    /// Use [`BinaneApi::subscribe()`] to start streaming data
    pub async fn connect(&mut self) -> crate::Result<()> {

        info!("Connecting to BinanceApi...");
        let (stream, _) = tokio_tungstenite::connect_async(APIURL).await?;
        self.stream.replace(stream);
        self.connected = true;
        info!("Connected!");

        Ok(())
    }

    /// Disconnects the connection, does nothing if not connected.
    pub async fn disconnect(&mut self) {
        // call close if we have a socket, without failing if we have no socket
        if let Some(socket) = self.stream.as_mut() {
            let _ = socket
                .close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: std::borrow::Cow::Borrowed("Normal"),
                }))
                .await;
        }
    }

    /// Get the next message from the stream.
    /// TODO: Implement Error Types here and return result instead
    pub async fn next_message(&mut self) -> Option<Message> {
        // gets the stream, if there are no stream, return None, no next message.
        let stream = self.stream.as_mut()?;

        loop {
            match stream.next().await? {
                Ok(msg) => {
                    match msg {
                        tungstenite::Message::Text(s) => {
                            let Ok(msg) = serde_json::from_str::<Message>(&s) else {
                                warn!("could not parse message {s:#?}");
                                continue;
                            };
                            return Some(msg);
                        }
                        tungstenite::Message::Ping(vec) => {
                            info!("Received Ping, sending Pong.");
                            let _ = stream.send(tungstenite::Message::Pong(vec)).await;
                        }

                        tungstenite::Message::Pong(vec) => {
                            info!("Received Pong, sending Ping.");
                            let _ = stream.send(tungstenite::Message::Ping(vec)).await;
                        }

                        tungstenite::Message::Close(close_frame) => {
                            self.connected = false;
                            // Should return none on next iteration
                            warn!("Close frame recieved from server: {close_frame:?}");
                        }

                        tungstenite::Message::Binary(_vec) => unimplemented!("binary recieved"),
                        tungstenite::Message::Frame(_frame) => unimplemented!("Frame recieved"),
                    }
                }
                Err(e) => {
                    // We may need to handle  to many messgaes errors here,
                    // but should probably not be a problem
                    error!("Error when calling next() on stream: {e}");
                    return None;
                }
            }
        }
    }

    /// Request to subscribe to [`Symbol`]s.
    /// This function returns nothing, listen
    /// to [`BinanceApi::next_message()`] for confirmation
    ///
    /// **Recommendation** Subscribe to all your symbols and feeds in one go,
    /// binance have a limit on how fast requests can be sent.

    /// This method will nest the request and does **not** throttle the events,
    /// therefore its up to you to not go over the binance request limit.
    ///
    /// Does nothing if an empty iterator supplied.
    pub async fn subscribe(&mut self, symbols: &[SubscribeInfo], id: Option<u32>) {
        if symbols.is_empty() {
            warn!("you must provide SubsribeInfo for atleast one Symbol");
            return;
        }

        let symbols: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@{}", s.symbol, s.feed))
            .collect();

        // Safe to unwrap since we use or
        let id = id.unwrap_or(1);

        let sub_string = format!(
            r#"{{"method":"SUBSCRIBE",
            "params": {symbols:?},
            "id": {id}
            }}"#
        );

        if let Err(e) = self
            .stream
            .as_mut()
            .expect("Not connected, you need to connect before subscribing")
            .send(tungstenite::Message::Text(sub_string))
            .await
        {
            error!("Error when Subscribing: {e}");
        }
    }

    /// Unsubscribe from [`Symbol`]s.
    ///
    /// Does nothing if no symbols are supplied,
    /// or if you are not subscribed to the provided Symbol(s)
    pub async fn unsubscribe(&mut self, symbols: Vec<SubscribeInfo>) {
        if symbols.is_empty() {
            warn!("you must provide SubsribeInfo for atleast one Symbol");
            return;
        }

        let symbols: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@{}", s.symbol, s.feed))
            .collect();

        let sub_string = format!(
            r#"{{"method":"UNSUBSCRIBE",
            "params": {symbols:?},
            "id": 1
            }}"#
        );

        if let Some(stream) = self.stream.as_mut() {
            let _ = stream.send(tungstenite::Message::Text(sub_string)).await;
        }
    }
}

/// Information required to subscribe to a feed for a Symbol.
pub struct SubscribeInfo {
    symbol: Symbol,
    feed: Feed,
}

impl SubscribeInfo {
    pub fn new(symbol: Symbol, feed: Feed) -> Self {
        Self { symbol, feed }
    }
}

/// Represents the available feeds for streaming data.
///
/// Each variant specifies a type of feed and its update behavior.
/// **Official docs:** see [the market-stream docs](https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams) for more details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Feed {
    /// The Aggregate Trade Streams provide aggregated trade information for a single taker order.
    ///
    /// **Update Speed:** Real-time
    ///
    /// Emits [`messages::AggTrade`] as part of the [`Message`] enum.
    AggTrade,

    /// The Trade Streams push raw trade information; each trade has a unique buyer and seller.
    /// Update Speed: Real-time
    /// Emits   TODO:
    Trade,

    /// Updateting BBO in realtime
    /// Pushes any update to the best bid or ask's price or quantity in real-time for a specified symbol.
    ///
    /// NOTE: does this indicate that partialdepth shouldnt be subscribed to over on connection?
    /// Multiple <symbol>@bookTicker streams can be subscribed to over one connection.
    ///
    /// **Update Speed:** 1ms
    BookTicker,

    /// # Partial Book Depth Streams
    /// Payload [`DepthLevel`]: Top bids and asks
    /// Valid are 5, 10, or 20.
    /// and [`Delay`], time between updates.
    /// **Update Speed:** 1000ms or 100ms
    ///
    /// # Emits [`messages::PartialDepth`] as part of the [`Message`] enum.
    PartialDepth {
        levels: DepthLevel,
        delay: Delay, //Delay:
    },

    /// Order book price and quantity depth updates used to locally manage an order book.
    ///
    /// Update Speed: 1000ms or 100ms
    FullDepth,
}

impl std::fmt::Display for Feed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Feed::AggTrade => "aggTrade".into(),
            Feed::Trade => "trade".into(),
            Feed::PartialDepth { levels, delay } => format!("depth{levels}{delay}"),
            Feed::BookTicker => "bookTicker".into(),
            Feed::FullDepth => todo!(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthLevel(u8);
impl DepthLevel {
    pub const FIVE: Self = Self(5);
    pub const TEN: Self = Self(10);
    pub const TWENTY: Self = Self(20);
}

impl std::fmt::Display for DepthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Delay for different feeds in the Binance api, in milliseconds.
///
/// The specific [`Feed`] will have a
/// Delay parameter if you can set this for the particular feed.
///
/// # Panic
/// If you provide a non compatible delay,
/// [`BinanceApi`] will panic!
///
/// See docs for each feed for compatible Delays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delay {
    /// 100 Milliseconds
    ONEHUNDRED,
    /// 1000 Milliseconds
    ONETHOUSAND,
}

impl std::fmt::Display for Delay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Delay::ONEHUNDRED => "@100ms",
            Delay::ONETHOUSAND => "",
        };
        write!(f, "{}", s)
    }
}
