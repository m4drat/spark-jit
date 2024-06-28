pub extern "C" fn pow(a: i64, b: i64) -> i64 {
    a.pow(b as u32)
}

pub extern "C" fn factorial(n: i64) -> i64 {
    (1..=n).product()
}
