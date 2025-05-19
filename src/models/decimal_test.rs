#[cfg(test)]
mod tests {
    use crate::models::decimal::SqlxDecimal;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_sqlx_decimal_conversions() {
        // Test From<Decimal> for SqlxDecimal
        let decimal = Decimal::from_str("123.456").unwrap();
        let sqlx_decimal = SqlxDecimal::from(decimal);
        assert_eq!(sqlx_decimal.0, decimal);

        // Test From<SqlxDecimal> for Decimal
        let decimal2: Decimal = sqlx_decimal.into();
        assert_eq!(decimal, decimal2);
    }

    #[test]
    fn test_sqlx_decimal_deref() {
        let decimal = Decimal::from_str("123.456").unwrap();
        let sqlx_decimal = SqlxDecimal(decimal);
        
        // Test deref
        assert_eq!(*sqlx_decimal, decimal);
        
        // Test using decimal methods through deref
        assert_eq!(sqlx_decimal.round_dp(2).to_string(), "123.46");
    }

    #[test]
    fn test_sqlx_decimal_arithmetic() {
        let a = SqlxDecimal(Decimal::from_str("100.00").unwrap());
        let b = SqlxDecimal(Decimal::from_str("50.00").unwrap());
        
        // Test addition
        assert_eq!((a + b).0, Decimal::from_str("150.00").unwrap());
        
        // Test subtraction
        assert_eq!((a - b).0, Decimal::from_str("50.00").unwrap());
        
        // Test multiplication
        assert_eq!((a * b).0, Decimal::from_str("5000.00").unwrap());
        
        // Test division
        assert_eq!((a / b).0, Decimal::from_str("2").unwrap());
        
        // Test negation
        assert_eq!((-a).0, Decimal::from_str("-100.00").unwrap());
    }

    #[test]
    fn test_sqlx_decimal_display() {
        let decimal = Decimal::from_str("123.456").unwrap();
        let sqlx_decimal = SqlxDecimal(decimal);
        
        assert_eq!(sqlx_decimal.to_string(), "123.456");
    }
} 