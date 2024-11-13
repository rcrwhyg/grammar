use core::fmt;
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use winnow::{
    ascii::{digit1, multispace0, Caseless},
    combinator::{alt, delimited, opt, separated, separated_pair, trace},
    error::{ContextError, ErrMode, ParserError},
    prelude::*,
    stream::{AsBStr, AsChar, Compare, FindSlice, ParseSlice, Stream, StreamIsPartial},
    token::take_until,
};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum Num {
    Int(i64),
    Float(f64),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(Num),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn main() -> Result<()> {
    let s = r#"{
        "name": "John Doe",
        "age": 43,
        "is_student": false,
        "marks": [87.0, 90, -45.7, 67.9],
        "address": {
            "city": "New York",
            "zip": 10001
        }
    }"#;

    let input = &mut (&*s);
    let json = parse_json(input)
        .map_err(|e: ErrMode<ContextError>| anyhow!("Failed to parse JSON: {:?}", e));

    println!("{:#?}", json);

    Ok(())
}

fn parse_json<Input, Error>(input: &mut Input) -> PResult<JsonValue, Error>
where
    Input: StreamIsPartial
        + Stream
        + Compare<char>
        + Compare<&'static str>
        + Compare<Caseless<&'static str>>
        + AsBStr
        + FindSlice<char>,
    <Input as Stream>::Token: AsChar + Clone,
    <Input as Stream>::Slice: fmt::Display + ParseSlice<f64> + ParseSlice<i64> + ParseSlice<bool>,
    <Input as Stream>::IterOffsets: Clone,
    Error: ParserError<Input>,
{
    parse_value(input)
}

fn skip_whitespace<Input, Output, Error, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<Input, (), Error>
where
    Input: Stream + StreamIsPartial,
    <Input as Stream>::Token: AsChar + Clone,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
{
    trace("skip_whitespace", move |input: &mut Input| {
        let _ = multispace0(input)?;
        parser.parse_next(input)?;
        multispace0.parse_next(input)?;
        Ok(())
    })
}

fn parse_null<Input, Error>(input: &mut Input) -> PResult<(), Error>
where
    Input: StreamIsPartial + Stream + Compare<&'static str>,
    Error: ParserError<Input>,
{
    "null".value(()).parse_next(input)
}

fn parse_bool<Input, Error>(input: &mut Input) -> PResult<bool, Error>
where
    Input: StreamIsPartial + Stream + Compare<&'static str>,
    <Input as Stream>::Slice: ParseSlice<bool>,
    Error: ParserError<Input>,
{
    // alt(("true".value(true), "false".value(false))).parse_next(input)
    alt(("true", "false")).parse_to().parse_next(input)
}

fn parse_num<Input, Error>(input: &mut Input) -> PResult<Num, Error>
where
    Input: StreamIsPartial
        + Stream
        + Compare<&'static str>
        + Compare<Caseless<&'static str>>
        + Compare<char>
        + AsBStr,
    <Input as Stream>::Slice: ParseSlice<i64> + ParseSlice<f64>,
    <Input as Stream>::Token: AsChar + Clone,
    <Input as Stream>::IterOffsets: Clone,
    Error: ParserError<Input>,
{
    // process the sign
    let sign = opt("-").map(|s| s.is_some()).parse_next(input)?;
    let num = digit1.parse_to::<i64>().parse_next(input)?;
    let ret: Result<(), ErrMode<ContextError>> = ".".value(()).parse_next(input);
    if ret.is_ok() {
        let frac = digit1.parse_to::<i64>().parse_next(input)?;
        let v = format!("{}.{}", num, frac).parse::<f64>().unwrap();
        Ok(if sign {
            Num::Float(-v as _)
        } else {
            Num::Float(v as _)
        })
    } else {
        Ok(if sign { Num::Int(-num) } else { Num::Int(num) })
    }
}

// this is too complicated for a single line parser, so we use `float` directly.
// fn parse_number<Input, Error>(input: &mut Input) -> PResult<f64, Error>
// where
//     Input: StreamIsPartial + Stream + Compare<Caseless<&'static str>> + AsBStr + Compare<char>,
//     <Input as Stream>::Slice: ParseSlice<f64>,
//     <Input as Stream>::Token: AsChar + Clone,
//     <Input as Stream>::IterOffsets: Clone,
//     Error: ParserError<Input>,
// {
//     float.parse_next(input)
// }

// json allows quoted strings to have escaped characters, so we need to handle that, but we won't do that here
fn parse_string<Input, Error>(input: &mut Input) -> PResult<String, Error>
where
    Input: StreamIsPartial + Stream + Compare<char> + FindSlice<char>,
    <Input as Stream>::Token: AsChar + Clone,
    <Input as Stream>::Slice: fmt::Display,
    Error: ParserError<Input>,
{
    let ret = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(ret.to_string())
}

fn parse_array<Input, Error>(input: &mut Input) -> PResult<Vec<JsonValue>, Error>
where
    Input: StreamIsPartial
        + Stream
        + Compare<char>
        + Compare<&'static str>
        + Compare<Caseless<&'static str>>
        + AsBStr
        + FindSlice<char>,
    <Input as Stream>::Token: AsChar + Clone,
    <Input as Stream>::Slice: fmt::Display + ParseSlice<f64> + ParseSlice<i64> + ParseSlice<bool>,
    <Input as Stream>::IterOffsets: Clone,
    Error: ParserError<Input>,
{
    let sep1 = skip_whitespace('[');
    let sep2 = skip_whitespace(']');
    let sep_comma = skip_whitespace(',');
    let parse_values = separated(1.., parse_value, sep_comma);
    delimited(sep1, parse_values, sep2).parse_next(input)
}

fn parse_object<Input, Error>(input: &mut Input) -> PResult<HashMap<String, JsonValue>, Error>
where
    Input: StreamIsPartial
        + Stream
        + Compare<char>
        + Compare<&'static str>
        + Compare<Caseless<&'static str>>
        + AsBStr
        + FindSlice<char>,
    <Input as Stream>::Token: AsChar + Clone,
    <Input as Stream>::Slice: fmt::Display + ParseSlice<f64> + ParseSlice<i64> + ParseSlice<bool>,
    <Input as Stream>::IterOffsets: Clone,
    Error: ParserError<Input>,
{
    let sep1 = skip_whitespace('{');
    let sep2 = skip_whitespace('}');
    let sep_comma = skip_whitespace(',');
    let sep_colon = skip_whitespace(':');
    let parse_kv_pair = separated_pair(parse_string, sep_colon, parse_value);
    let parse_kv = separated(1.., parse_kv_pair, sep_comma);
    delimited(sep1, parse_kv, sep2).parse_next(input)
}

fn parse_value<Input, Error>(input: &mut Input) -> PResult<JsonValue, Error>
where
    Input: StreamIsPartial
        + Stream
        + Compare<char>
        + Compare<&'static str>
        + Compare<Caseless<&'static str>>
        + AsBStr
        + FindSlice<char>,
    <Input as Stream>::Token: AsChar + Clone,
    <Input as Stream>::Slice: fmt::Display + ParseSlice<f64> + ParseSlice<i64> + ParseSlice<bool>,
    <Input as Stream>::IterOffsets: Clone,
    Error: ParserError<Input>,
{
    alt((
        parse_null.value(JsonValue::Null),
        parse_bool.map(JsonValue::Bool),
        parse_num.map(JsonValue::Number),
        // parse_number.map(JsonValue::Number),
        parse_string.map(JsonValue::String),
        parse_array.map(JsonValue::Array),
        parse_object.map(JsonValue::Object),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() -> PResult<(), ContextError> {
        let s = "null";
        let input = &mut (&*s);
        parse_null(input)?;

        Ok(())
    }

    #[test]
    fn test_parse_bool() -> PResult<(), ContextError> {
        let s = "true";
        let input = &mut (&*s);
        let result = parse_bool(input)?;
        assert!(result);

        let s = "false";
        let input = &mut (&*s);
        let result = parse_bool(input)?;
        assert!(!result);

        Ok(())
    }

    #[test]
    fn test_parse_num() -> PResult<(), ContextError> {
        let s = "123";
        let input = &mut (&*s);
        let result = parse_num(input)?;
        assert_eq!(result, Num::Int(123));

        let s = "-123";
        let input = &mut (&*s);
        let result = parse_num(input)?;
        assert_eq!(result, Num::Int(-123));

        let s = "123.45";
        let input = &mut (&*s);
        let result = parse_num(input)?;
        assert_eq!(result, Num::Float(123.45));

        let s = "-123.45";
        let input = &mut (&*s);
        let result = parse_num(input)?;
        assert_eq!(result, Num::Float(-123.45));

        Ok(())
    }

    #[test]
    fn test_parse_string() -> PResult<(), ContextError> {
        let s = r#""hello""#;
        let input = &mut (&*s);
        let result = parse_string(input)?;
        assert_eq!(result, "hello".to_string());

        Ok(())
    }

    #[test]
    fn test_parse_array() -> PResult<(), ContextError> {
        let s = r#"[1, 2, 3]"#;
        let input = &mut (&*s);
        let result = parse_array(input)?;

        assert_eq!(
            result,
            vec![
                JsonValue::Number(Num::Int(1)),
                JsonValue::Number(Num::Int(2)),
                JsonValue::Number(Num::Int(3))
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_object() -> PResult<(), ContextError> {
        let s = r#"{"name": "John Doe", "age": 43}"#;
        let input = &mut (&*s);
        let result = parse_object(input)?;
        let mut map = HashMap::new();
        map.insert(
            "name".to_string(),
            JsonValue::String("John Doe".to_string()),
        );
        map.insert("age".to_string(), JsonValue::Number(Num::Int(43)));
        assert_eq!(result, map);

        Ok(())
    }

    #[test]
    fn test_parse_value() -> PResult<(), ContextError> {
        let s = r#""hello""#;
        let input = &mut (&*s);
        let result = parse_value(input)?;
        assert_eq!(result, JsonValue::String("hello".to_string()));

        Ok(())
    }
}
