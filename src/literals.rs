use std::fmt;

/// A concrete value of an atomic type.
#[derive(PartialEq, Debug)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
}

impl LiteralValue {
    #[allow(dead_code)]  // TODO!
    fn serialize(&self) -> String {
        match self {
            LiteralValue::Int(i) => i.to_string(),
            LiteralValue::Float(f) => format!("{:-?}", f),
            LiteralValue::String(s) => format!("\"{}\"", s),
        }
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "{:?}", self)
    }
}

pub mod parsing {
    use std::convert::Infallible;

    use nom::branch::alt;
    use nom::bytes::complete::tag;
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
        map_res(recognize((opt(one_of("+-")), decimal)), make_int).parse(input)
    }

    fn make_float(input: &str) -> Result<LiteralValue, std::num::ParseFloatError> {
        match str::parse::<f64>(input) {
            Ok(x) => Ok(LiteralValue::Float(x)),
            Err(x) => Err(x),
        }
    }

    // Adapted from https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#floating-point-numbers
    fn float_grammar(input: &str) -> IResult<&str, &str> {
        recognize((
            opt(one_of("+-")),
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
                // Special cases
                tag("inf"),
                tag("NaN"),
            ))
        )).parse(input)
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

    /// Parse a literal of one of the atomic types (int, float, string); returns the remaining
    /// string and the LiteralValue in question.
    pub fn literal(input: &str) -> IResult<&str, LiteralValue> {
        alt((float, int, string)).parse(input)
    }
}

#[cfg(test)]
pub mod parsing_sample_data {
    use rand::distr::StandardUniform;
    use rand::{Rng, SeedableRng, rngs::SmallRng};

    pub fn example_ints(how_many: usize) -> Vec<i64> {
        let mut how_many = how_many;
        let basic_examples: Vec<i64> = vec![1, 0, i64::MAX, -1, i64::MIN, 100, 1000, -100];
        let mut result: Vec<i64> = basic_examples.iter().take(how_many).cloned().collect();
        how_many -= result.len();
        if how_many > 0 {
            let rng = SmallRng::seed_from_u64(42);
            result.extend(rng.random_iter::<i64>().take(how_many));
        }
        result
    }

    pub fn example_floats(how_many: usize) -> Vec<f64> {
        let mut how_many = how_many;
        let basic_examples: Vec<f64> = vec![
            1.0, 0.0, -1.0, 1e14, 1e-14, -0.0,
            f64::MAX, f64::MIN_POSITIVE, f64::MIN, -f64::MIN_POSITIVE,
            f64::INFINITY, f64::NEG_INFINITY, f64::NAN,
            /* TODO:  Non-canonical NAN values */];
        let mut result: Vec<f64> = basic_examples.iter().take(how_many).cloned().collect();
        how_many -= result.len();
        if how_many > 0 {
            let rng = SmallRng::seed_from_u64(42);
            result.extend(rng.random_iter::<f64>().take(how_many));
        }
        result
    }

    pub fn example_strings(how_many: usize) -> Vec<String> {
        let mut how_many = how_many;
        let basic_examples: Vec<&str> = vec![
            "abc", "a", "100",
            "\"", "\'", "\\", "",
            "\\", "\\\\", "\\\\\\"
        ];
        let mut result: Vec<String> = basic_examples.iter().take(how_many)
            .map(|x| str::to_string(x)).collect();
        how_many -= result.len();
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..how_many {
            let len: usize = rng.random_range(1..10);
            let chars_to_add: Vec<char> = rng.clone().sample_iter(StandardUniform).take(len).collect();
            result.push(chars_to_add.into_iter().collect());
        };
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{parsing_sample_data::{example_floats, example_ints, example_strings}, *};

    #[test]
    fn smoke_test() {
        assert_eq!(parsing::literal("7"), Ok(("", LiteralValue::Int(7))));
        assert_eq!(parsing::literal(".7"), Ok(("", LiteralValue::Float(0.7))));
        assert_eq!(
            parsing::literal("\"foo\""),
            Ok(("", LiteralValue::String("foo".to_string())))
        );
    }

    #[test]
    fn int_round_trip() {
        for i in example_ints(100) {
            assert_eq!(Ok(("", LiteralValue::Int(i))),
                       parsing::literal(&LiteralValue::Int(i).serialize()))
        }
    }

    // Floats are a bit trickier because (1) NaN != NaN and (2) errors are common enough that we
    // want good debug output.
    #[test]
    fn float_round_trip() {
        for f in example_floats(100) {
            let serialized = LiteralValue::Float(f).serialize();
            let (remainder, deserialized) = parsing::literal(&serialized).unwrap();
            assert!(remainder == "");  // Parse should be total.
            match deserialized {
                LiteralValue::Float(result) => {
                    if f.is_nan() {
                        assert!(result.is_nan())
                    } else {
                        assert_eq!(f, result,
                                "Testing parse of {} serialized as {}", f, serialized);        
                    }
                }
                _ => { assert!(false, "deserialize {} -> {} -> {} was not float",
                               f, serialized, deserialized)}
            }
        }
    }

    #[test]
    #[should_panic]  // TODO:  Fix string serialization to make this pass.
    fn string_round_trip() {
        for s in example_strings(100) {
            assert_eq!(Ok(("", LiteralValue::String(s.clone()))),
                       parsing::literal(&LiteralValue::String(s.clone()).serialize()))
        }
    }
}
