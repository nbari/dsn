pub enum ParseError {}

pub fn parse(input: &str) -> Result<DSN, ::ParseError> {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
