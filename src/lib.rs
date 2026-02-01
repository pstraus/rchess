// Library crate for rchess — small, testable APIs for the binary.

// Generated protobuf code.
pub mod rchess {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/rchess.v1.rs"));
    }
}

pub mod pieces;
pub mod board;

/// Return a short greeting string. Kept minimal so unit tests are easy.
pub fn greet() -> String {
    "Hello, world!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_expected() {
        assert_eq!(greet(), "Hello, world!");
    }
}
