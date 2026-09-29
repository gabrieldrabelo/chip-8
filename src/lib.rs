pub fn hello() -> &'static str {
    "hello from chip8"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_works() {
        assert_eq!(hello(), "hello from chip8");
    }
}
