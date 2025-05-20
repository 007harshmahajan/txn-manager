use criterion::{criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;
use std::str::FromStr;

// Note: These imports would work if this benchmark file
// is properly set up with the transaction manager crate
// For now, commenting them out as they're for illustrative purposes
// use txn_manager::models::decimal::SqlxDecimal;
// use txn_manager::models::transaction::{TransactionType, TransactionStatus};

// Simple decimal conversion benchmarks
fn decimal_conversion_benchmark(c: &mut Criterion) {
    let decimal_strings = [
        "123.456",
        "0.00001",
        "9999999.99999",
        "0.0",
        "1000000.0"
    ];
    
    c.bench_function("decimal_from_str", |b| {
        b.iter(|| {
            for s in &decimal_strings {
                let _ = Decimal::from_str(s).unwrap();
            }
        })
    });
    
    // This benchmark would test our custom SqlxDecimal conversion
    // when properly set up with the crate
    /*
    c.bench_function("decimal_to_sqlx_decimal", |b| {
        let decimals: Vec<Decimal> = decimal_strings
            .iter()
            .map(|s| Decimal::from_str(s).unwrap())
            .collect();
            
        b.iter(|| {
            for d in &decimals {
                let _ = SqlxDecimal::from(*d);
            }
        })
    });
    */
}

// Simple formatting benchmark
fn decimal_formatting_benchmark(c: &mut Criterion) {
    let decimal = Decimal::from_str("1234567.89").unwrap();
    
    c.bench_function("decimal_to_string", |b| {
        b.iter(|| decimal.to_string())
    });
}

criterion_group!(
    benches, 
    decimal_conversion_benchmark,
    decimal_formatting_benchmark
);
criterion_main!(benches); 