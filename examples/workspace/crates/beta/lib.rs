pub fn mul(a: i64, b: i64) -> i64 { a * b }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn it_muls() { assert_eq!(mul(2, 3), 6); }
}
