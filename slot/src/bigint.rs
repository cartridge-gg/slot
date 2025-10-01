use num_bigint::BigInt as NumBigInt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct BigInt(NumBigInt);

impl FromStr for BigInt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NumBigInt::from_str(s)
            .map(BigInt)
            .map_err(|_| format!("Failed to parse BigInt from string: {}", s))
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_from_str() {
        let input = "123";
        let result = BigInt::from_str(input);
        assert!(result.is_ok(), "BigInt::from_str failed on '{}'", input);
    }
}
