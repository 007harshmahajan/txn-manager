use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::decode::Decode;
use sqlx::encode::{Encode, IsNull};
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::types::BigDecimal;
use sqlx::Type;
use std::fmt;
use std::ops::{Add, Deref, DerefMut, Div, Mul, Neg, Sub};
use std::str::FromStr;

/// A wrapper around rust_decimal::Decimal to implement SQLx traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SqlxDecimal(pub Decimal);

// Implement Deref and DerefMut so we can use SqlxDecimal like a Decimal
impl Deref for SqlxDecimal {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SqlxDecimal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Implement From/Into for easy conversion
impl From<Decimal> for SqlxDecimal {
    fn from(decimal: Decimal) -> Self {
        SqlxDecimal(decimal)
    }
}

impl From<SqlxDecimal> for Decimal {
    fn from(sql_decimal: SqlxDecimal) -> Self {
        sql_decimal.0
    }
}

// Implement Display
impl fmt::Display for SqlxDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement operator traits for convenience
impl Add for SqlxDecimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        SqlxDecimal(self.0 + rhs.0)
    }
}

impl Sub for SqlxDecimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        SqlxDecimal(self.0 - rhs.0)
    }
}

impl Mul for SqlxDecimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        SqlxDecimal(self.0 * rhs.0)
    }
}

impl Div for SqlxDecimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        SqlxDecimal(self.0 / rhs.0)
    }
}

impl Neg for SqlxDecimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        SqlxDecimal(-self.0)
    }
}

// Add a conversion from BigDecimal to SqlxDecimal
impl From<BigDecimal> for SqlxDecimal {
    fn from(value: BigDecimal) -> Self {
        // Convert from BigDecimal to String to Decimal
        let decimal_str = value.to_string();
        SqlxDecimal(Decimal::from_str(&decimal_str).unwrap_or_default())
    }
}

// Implement SQLx traits for our wrapper type
impl<'q> Encode<'q, sqlx::Postgres> for SqlxDecimal {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // Convert to BigDecimal first, which implements Encode for Postgres
        let big_decimal = BigDecimal::from_str(&self.0.to_string()).unwrap_or_default();
        <BigDecimal as Encode<sqlx::Postgres>>::encode_by_ref(&big_decimal, buf)
    }
}

impl<'r> Decode<'r, sqlx::Postgres> for SqlxDecimal {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        // First try to decode as BigDecimal (PostgreSQL's NUMERIC type)
        match BigDecimal::decode(value.clone()) {
            Ok(bigdec) => {
                let decimal_str = bigdec.to_string();
                match Decimal::from_str(&decimal_str) {
                    Ok(decimal) => Ok(SqlxDecimal(decimal)),
                    Err(_) => Ok(SqlxDecimal(Decimal::ZERO)),
                }
            }
            Err(_) => {
                // If that fails, fall back to string parsing
                let bytes = <&[u8] as Decode<sqlx::Postgres>>::decode(value)?;
                let s = std::str::from_utf8(bytes)?;
                match Decimal::from_str(s) {
                    Ok(dec) => Ok(SqlxDecimal(dec)),
                    Err(_) => Ok(SqlxDecimal(Decimal::ZERO)), // Last resort, use zero
                }
            }
        }
    }
}

impl Type<sqlx::Postgres> for SqlxDecimal {
    fn type_info() -> PgTypeInfo {
        // Use BigDecimal's type info since it maps to Postgres NUMERIC
        <BigDecimal as Type<sqlx::Postgres>>::type_info()
    }
}
