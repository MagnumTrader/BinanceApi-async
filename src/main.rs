mod historical;

use binance_api_async::{
    messages, BinanceApi, Delay, DepthLevel, Feed, Message, SubscribeInfo, Symbol,
};

use tokio::time::MissedTickBehavior;
use tracing::{error, info};

type Result<T> = std::result::Result<T, binance_api_async::Error>;

#[tokio::main]
pub async fn main() -> Result<()> {

    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();

    // define the feeds i want to subscribe to
    let ob = Feed::PartialDepth {
        levels: DepthLevel::FIVE,
        delay: Delay::ONEHUNDRED,
    };
    let trade = Feed::AggTrade;

    let symbols = vec![
        SubscribeInfo::new(Symbol::BTCUSDT, trade),
        SubscribeInfo::new(Symbol::BTCUSDT, ob)
    ];

    let mut api = BinanceApi::new();
    api.connect().await?;

    // set a timer for every 24 hours so that we refresh the connection to Binance.
    let mut reconnection_timer = tokio::time::interval(std::time::Duration::from_secs(86400));
    reconnection_timer.set_missed_tick_behavior(MissedTickBehavior::Burst);
    reconnection_timer.tick().await;

    api.subscribe(&symbols, None).await;

    loop {
        tokio::select! {
            msg = api.next_message() => {
                match msg {
                    // send this to some db writer thread
                    // we should get some kind of Binance::Message with the variants
                    Some(msg) => {
                        match msg {
                            Message::AggTrade(_at) => {}
                            Message::PartialDepth(pd)=>{},
                            Message::BookTicker(bt) => {println!("{bt:?}")}
                            Message::SubscribeSuccess { .. } => {info!("Successfully subscribed!")},
                        }
                    },
                    None => {
                        info!("Api as disconnected, trying to reconnect");
                        try_reconnect(&mut api, &symbols).await.expect("expect to be able to reconnect");
                    }
                }
            }
            _ = reconnection_timer.tick() => {
                info!("Timeout, reconnecting!");
                try_reconnect(&mut api, &symbols).await.expect("should be able to reconnect");
            }
        }
    }
    #[allow(unreachable_code)]
    Ok(())
}

/// Function to attempt reconnections
pub async fn try_reconnect(api: &mut BinanceApi, symbols: &[SubscribeInfo]) -> Result<()> {
    let mut attempts = 0;

    // sending after closing is not allowed
    api.disconnect().await;
    while let Err(x) = api.connect().await {
        attempts += 1;
        error!("reconnection attempt {attempts}, error occured when reconnecting {x}");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        if attempts > 12 {
            return Err(x);
        }
    }
    info!("Successfully reconnected!");
    info!("Subscribing...");
    api.subscribe(symbols, None).await;

    Ok(())
}

const CLEAR: &str = "\x1B[2J\x1B[1;1H";

fn display_ob(book: &messages::PartialDepth) {
    let (best_bid, best_ask) = (book.bids.first().unwrap()[0], book.asks.first().unwrap()[0]);

    let midprice = (best_bid + best_ask) / rust_decimal::Decimal::TWO;
    let dollar_spread = best_ask - best_bid;
    let spread = dollar_spread / best_ask * rust_decimal::Decimal::ONE_HUNDRED;

    let bids = book.bids.iter();
    let asks = book.asks.iter();

    print!("{CLEAR}");

    println!(" Mid Price: {midprice} Dollar spread: {dollar_spread} Spread: {spread:.3}% ");

    for (bid, ask) in bids.zip(asks) {
        println!(
            "{bidvolume:.5} {bidprice:.5} - {askprice:.5} {askvolume:.5}",
            bidprice = bid[0],
            bidvolume = bid[1],
            askprice = ask[0],
            askvolume = ask[1]
        );
    }
}
