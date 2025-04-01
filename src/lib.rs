/// The "real" main entry point of our binary target, relocated to a lib so that
/// rust-test can reach it.
pub fn _main() {
    println!("Hello, World!");
}


#[cfg(test)]
mod tests {
    use super::*;

    /// For illustrative purposes:  A trivial smoke test.
    #[test]
    fn smoke_test() {
        _main();
    }
}
