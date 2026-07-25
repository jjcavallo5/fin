use rust_decimal::{Decimal, RoundingStrategy};
use std::str::FromStr;

pub fn parse_dollars_to_cents(value: &str) -> Result<i64, String> {
    let decimal = Decimal::from_str(value.trim())
        .or_else(|_| Decimal::from_scientific(value.trim()))
        .map_err(|error| format!("invalid dollar amount: {error}"))?;
    let cents = (decimal * Decimal::from(100))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    cents
        .to_string()
        .parse::<i64>()
        .map_err(|error| format!("dollar amount is out of range: {error}"))
}

pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.unsigned_abs();
    format!("{sign}${}.{:02}", absolute / 100, absolute % 100)
}

#[cfg(test)]
mod tests {
    use super::parse_dollars_to_cents;

    #[test]
    fn parses_and_rounds_dollars() {
        assert_eq!(parse_dollars_to_cents("1234.56").unwrap(), 123_456);
        assert_eq!(parse_dollars_to_cents("10.105").unwrap(), 1_011);
        assert_eq!(parse_dollars_to_cents("-0.005").unwrap(), -1);
    }
}
