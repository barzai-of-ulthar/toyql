#[derive(PartialEq, Debug)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
}

pub mod parsing {
    use std::convert::Infallible;

    use nom::branch::alt;
    use nom::character::complete::{anychar, char, none_of, one_of};
    use nom::combinator::{map_res, opt, recognize};
    use nom::multi::{many0, many1};
    use nom::sequence::preceded;
    use nom::sequence::terminated;
    use nom::IResult;
    use nom::Parser;

    use super::LiteralValue;

    fn make_int(input: &str) -> Result<LiteralValue, std::num::ParseIntError> {
        match str::parse::<i64>(input) {
            Ok(x) => Ok(LiteralValue::Int(x)),
            Err(x) => Err(x),
        }
    }

    fn int(input: &str) -> IResult<&str, LiteralValue> {
        map_res(recognize(decimal), make_int).parse(input)
    }

    fn make_float(input: &str) -> Result<LiteralValue, std::num::ParseFloatError> {
        match str::parse::<f64>(input) {
            Ok(x) => Ok(LiteralValue::Float(x)),
            Err(x) => Err(x),
        }
    }

    // Adapted from https://github.com/rust-bakery/nom/blob/main/Cargo.toml
    fn float_grammar(input: &str) -> IResult<&str, &str> {
        alt((
            // Case one: .42
            recognize((
                char('.'),
                decimal,
                opt((one_of("eE"), opt(one_of("+-")), decimal)),
            )), // Case two: 42e42 and 42.42e42
            recognize((
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal,
            )), // Case three: 42. and 42.42
            recognize((decimal, char('.'), opt(decimal))),
        ))
        .parse(input)
    }

    fn decimal(input: &str) -> IResult<&str, &str> {
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))).parse(input)
    }

    fn float(input: &str) -> IResult<&str, LiteralValue> {
        map_res(recognize(float_grammar), make_float).parse(input)
    }

    fn make_string(input: &str) -> Result<LiteralValue, Infallible> {
        match str::parse::<String>(input) {
            Ok(x) => {
                let without_quotes = x.trim_matches('\"').to_string();
                let without_backslash = without_quotes.replace("\\", "");
                Ok(LiteralValue::String(without_backslash))
            }
            Err(x) => Err(x),
        }
    }

    fn string(input: &str) -> IResult<&str, LiteralValue> {
        let string_grammar = recognize((
            char('"'),
            many0(alt((
                recognize((char('\\'), anychar)),
                recognize(many1(none_of("\\\""))),
            ))),
            char('"'),
        ));
        map_res(recognize(string_grammar), make_string).parse(input)
    }

    pub fn literal(input: &str) -> IResult<&str, LiteralValue> {
        alt((float, int, string)).parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        assert_eq!(parsing::literal("7"), Ok(("", LiteralValue::Int(7))));
        assert_eq!(parsing::literal(".7"), Ok(("", LiteralValue::Float(0.7))));
        assert_eq!(
            parsing::literal("\"foo\""),
            Ok(("", LiteralValue::String("foo".to_string())))
        );
    }
}
