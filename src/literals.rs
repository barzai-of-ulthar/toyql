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

    /// Check that `self` and `other` represent identical values, regardless of their equality
    /// relation (i.e. with nan==nan semantics).
    // TODO(barzai) This should also handle unicode polysemy!
    #[allow(dead_code)]  // TODO!
    fn identical(&self, other: &LiteralValue) -> bool {
        if let LiteralValue::Float(l) = self {
            if let LiteralValue::Float(r ) = other {
                if l.is_nan() && r.is_nan() { return true; }
            }
        };
        self == other
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

    pub mod string {
        use nom::branch::alt;
        use nom::bytes::complete::{escaped_transform, tag};
        use nom::combinator::{opt, value};
        use nom::character::complete::{char, none_of};
        use nom::combinator::map_res;
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
        // (`escaped_transform`) but unlike other bits of nom it can't handle  zero-length
        // strings.  So we have to handle the representation part of this match a bit more care.
        //
        // So be very careful editing this code.  The test cases here are *essential* and the
        // errors are *wicked*.

        fn make_repr(input: Option<String>) -> Result<LiteralValue, std::string::ParseError> {
            match input {
                Some(s) => {
                    match str::parse::<String>(&s) {
                        Ok(x) => {
                            Ok(LiteralValue::String(x))
                        }
                        Err(x) => Err(x),
                   }
                }
                None => Ok(LiteralValue::String("".to_string()))
            }
        }

        pub fn apply_grammar(input: &str) -> IResult<&str, LiteralValue> {
            let string_grammar = 
                delimited(
                    char('"'),
                    opt(escaped_transform(
                        // A string encoding must exclude at least two points:  A record separator and
                        // an escape character.
                        none_of("\\\""),
                        '\\',
                        alt((
                            // Read value(val, parser) as:
                            //    The string `val` is is produced when the parse `parser` matches
                            //    after an escape.
                            value("\\", tag("\\")),
                            value("\"", tag("\"")),
                            value("\n", tag("n")),
                        )))),
                    char('"')
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
    use itertools::Itertools;
    use crate::test_utils::{example_ints, example_floats, example_strings};
    use super::parsing::{int, float, string};
    
    // TODO(barzai) This only returns canonical forms, not weird stuff.
    pub fn example_literal_representations(how_many: usize) -> Vec<String> {
        example_ints(how_many).iter().map(|i| int::serialize(*i))
            .interleave(
                example_floats(how_many).iter().map(|f| float::serialize(*f))
            )
            .interleave(
                example_strings(how_many).iter().map(|s| string::serialize(s))
            ).take(how_many).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::literals::{parsing, LiteralValue};

    use super::parsing_sample_data::example_literal_representations;
    use crate::test_utils::{example_ints, example_floats, example_strings};

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
                                   "Testing parse of {} serialized as {} deserializes to {}", f, serialized, deserialized);
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
                        "Testing parse of >{}< serialized as >{}< deserializes to >{}<", s, serialized, result);
                    }
                _ => { assert!(false, "deserialize {} -> {} -> {} was not string",
                               s, serialized, deserialized)}
            }
        }
    }

    #[test]
    fn literal_round_trip() {
        for serialized in example_literal_representations(100) {
            let (remainder, deserialized) = parsing::apply_grammar(&serialized).unwrap();
            assert!(remainder == "");  // Parse should be total.
            let reserialized = LiteralValue::serialize(&deserialized);
            let (remainder, re_deserialized) = parsing::apply_grammar(&reserialized).unwrap();
            assert!(remainder == "");  // Parse should be total.
            assert!(deserialized.identical(&re_deserialized));
        } 
    }
}
