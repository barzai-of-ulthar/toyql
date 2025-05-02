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
        parsing::serialize(self)
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "{:?}", self)
    }
}

pub mod parsing {
    use nom::branch::alt;
    use nom::character::complete::{char, one_of};
    use nom::combinator::recognize;
    use nom::multi::{many0, many1};
    use nom::sequence::terminated;
    use nom::IResult;
    use nom::Parser;

    use super::LiteralValue;

    // General-purpose utility to extract any number of decimal digits with rust-style separator
    // conventions.
    fn decimal(input: &str) -> IResult<&str, &str> {
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))).parse(input)
    }

    // For each type, we need to have both a grammar (something that identifies and extracts
    // text during parsing), an interpretation (something that turns the extracted text that
    // matches the grammar into a usable representation) and a serialization (a way of turning
    // that representation into something that satisfies the grammar).  For ease of understanding
    // we break out these triads into per-type namespaces.

    pub mod int {
        use nom::IResult;
        use nom::combinator::{map_res, opt, recognize};
        use nom::character::complete::one_of;
        use nom::Parser;

        use crate::literals::LiteralValue;

        pub fn make_repr(input: &str) -> Result<LiteralValue, std::num::ParseIntError> {
            match str::parse::<i64>(input) {
                Ok(x) => Ok(LiteralValue::Int(x)),
                Err(x) => Err(x),
            }
        }
    
        pub fn apply_grammar(input: &str) -> IResult<&str, LiteralValue> {
            map_res(recognize((opt(one_of("+-")), super::decimal)), make_repr).parse(input)
        }

        pub fn serialize(v: i64) -> String {
            v.to_string()
        }
    }

    pub mod float {
        use nom::branch::alt;
        use nom::bytes::complete::tag;
        use nom::character::complete::{char, one_of};
        use nom::combinator::{map_res, opt, recognize};
        use nom::sequence::preceded;
        use nom::IResult;
        use nom::Parser;

        use super::decimal;
        use crate::literals::LiteralValue;

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

        pub fn apply_grammar(input: &str) -> IResult<&str, LiteralValue> {
            map_res(recognize(float_grammar), make_repr).parse(input)
        }

        pub fn make_repr(input: &str) -> Result<LiteralValue, std::num::ParseFloatError> {
            match str::parse::<f64>(input) {
                Ok(x) => Ok(LiteralValue::Float(x)),
                Err(x) => Err(x),
            }
        }

        pub fn serialize(v: f64) -> String {
            format!("{:-?}", v)
        }
    }

    mod string {
        use nom::branch::alt;
        use nom::bytes::complete::{escaped_transform, tag};
        use nom::combinator::value;
        use nom::character::complete::{char, none_of};
        use nom::combinator::{map_res, recognize};
        use nom::sequence::delimited;
        use nom::IResult;
        use nom::Parser;
    
        use super::LiteralValue;
    
        // Strings are rather nastier than other types because of the combination of quoting,
        // escaping, and possible emptiness.  We need to do two things:  Extract the interior of
        // the quotes (which requires awareness of escaping so we don't use an escaped quote) and
        // convert escapes into their meaning.
        //
        // Nom provides a method to both recognize a string and expand its escapes
        // (`escaped_transform`) but I have been unable to make this work reliably within a
        // larger grammar or on zero-length strings.  So we have to handle the representation
        // part of this match a bit more care and do it in two phases:  First recognize the full
        // grammar and remove the quotes, then perform escape expansion on the interior.

        fn make_repr(input: &str) -> Result<LiteralValue, std::string::ParseError> {
            match str::parse::<String>(input) {
                Ok(x) => {
                    let without_quotes = x.trim_matches('\"').to_string();
                    Ok(LiteralValue::String(without_quotes))
                }
                Err(x) => Err(x),
            }
        }

        pub fn apply_grammar(input: &str) -> IResult<&str, LiteralValue> {
            println!("Attempting to apply the string grammar to ->{}<-", input);
            let string_grammar = recognize(
                delimited(
                    char('"'),
                    escaped_transform(
                        // A string encoding must exclude at least two points:  A record separator and
                        // an escape character.
                        none_of("\\\""),
                        '\\',
                        alt((
                            //  This string...  is produced by this parse matching after an escape.
                            value("\\", tag("\\")),
                            value("\"", tag("\"")),
                            value("\n", tag("n")),
                        ))),
                    char('"'))
            );
            map_res(string_grammar, make_repr).parse(input)
        }

        pub fn serialize(v: &str) -> String {
            format!("\"{}\"", v
                .replace("\\", "\\\\")
                .replace("\n", "\\n")
                .replace("\"", "\\\"")
            )
        }
    }

    /// Parse a literal of one of the atomic types (int, float, string); returns the remaining
    /// string and the LiteralValue in question.
    pub fn apply_grammar(input: &str) -> IResult<&str, LiteralValue> {
        alt((float::apply_grammar, int::apply_grammar, string::apply_grammar)).parse(input)
    }

    /// Turn a literal representation back into its serial form.
    pub fn serialize(v: &LiteralValue) -> String {
        match &v {
            LiteralValue::Int(i) => int::serialize(*i),
            LiteralValue::Float(f) => float::serialize(*f),
            LiteralValue::String(s) => string::serialize(s)
        }
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
            " \\ ",
            "\'", "\n", "\\",
            "\\\\", "\\\\\\",
            "\"", 
            "",
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
        assert_eq!(parsing::apply_grammar("7"), Ok(("", LiteralValue::Int(7))));
        assert_eq!(parsing::apply_grammar(".7"), Ok(("", LiteralValue::Float(0.7))));
        assert_eq!(
            parsing::apply_grammar("\"foo\""),
            Ok(("", LiteralValue::String("foo".to_string())))
        );
    }

    #[test]
    fn int_round_trip() {
        for i in example_ints(100) {
            assert_eq!(Ok(("", LiteralValue::Int(i))),
                       parsing::apply_grammar(&LiteralValue::Int(i).serialize()))
        }
    }

    // Floats are a bit trickier because (1) NaN != NaN and (2) errors are common enough that we
    // want good debug output.
    #[test]
    fn float_round_trip() {
        for f in example_floats(100) {
            let serialized = LiteralValue::Float(f).serialize();
            let (remainder, deserialized) = parsing::apply_grammar(&serialized).unwrap();
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
    fn string_round_trip() {
        for s in example_strings(100) {
            let serialized = LiteralValue::String(s.clone()).serialize();
            let (remainder, deserialized) = parsing::apply_grammar(&serialized).unwrap();
            assert!(remainder == "");  // Parse should be total.
            match deserialized {
                LiteralValue::String(result) => {
                    assert_eq!(s, result,
                               "Testing parse of {} serialized as {}", s, serialized);        
                }
                _ => { assert!(false, "deserialize {} -> {} -> {} was not string",
                               s, serialized, deserialized)}
            }
        }
    }
}
