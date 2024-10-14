pub extern "C" fn pow(a: i64, b: i64) -> i64 {
    a.wrapping_pow(b as u32)
}

pub extern "C" fn factorial(n: i64) -> i64 {
    let mut fact: i64 = 1;
    for i in 1..=n {
        fact = fact.wrapping_mul(i);
    }

    fact
}
