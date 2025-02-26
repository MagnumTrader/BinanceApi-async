use derive_more::From;
use tokio_tungstenite::tungstenite;

#[derive(Debug, From)]
pub enum Error {
    ReconnectionTimeout,
    WebSocketError(tungstenite::Error),
    Custom(String),
}
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_trait_test() {
        let ts_error = tungstenite::Error::AttackAttempt;

        let my_err: Error = ts_error.into();

        // Tungstenite Error does not implement Eq
        if let Error::WebSocketError(tungstenite::Error::AttackAttempt) = my_err {
            assert!(true)
        } else {
            assert!(false)
        }

    }
}
