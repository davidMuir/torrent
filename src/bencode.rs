use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt::Display, str};

pub fn decode_bencoded(input: &[u8]) -> Result<Vec<Decoded>> {
    let (decodeds, _) = parse_internal(input)?;

    Ok(decodeds)
}

fn parse_string(input: &[u8]) -> Result<(&[u8], usize)> {
    let colon_index = input
        .iter()
        .position(|v| v == &b':')
        .ok_or(anyhow!("Couldn't parse string"))?;
    let number: usize = str::from_utf8(&input[0..colon_index])?.parse()?;
    let end_index = colon_index + 1 + number;
    let string = &input[colon_index + 1..end_index];

    Ok((string, end_index))
}

fn parse_integer(input: &[u8]) -> Result<(i64, usize)> {
    let end_index = input
        .iter()
        .position(|v| v == &b'e')
        .ok_or(anyhow!("Couldn't parse integer - missing end (e)"))?;

    let number = str::from_utf8(&input[1..end_index])?.parse()?;

    Ok((number, end_index + 1))
}

fn parse_internal(input: &[u8]) -> Result<(Vec<Decoded>, usize)> {
    let mut current_index = 0;
    let mut decodeds = vec![];

    while current_index < input.len() {
        let current_value = input[current_index];

        if current_value.is_ascii_digit() {
            let (string, end_index) = parse_string(&input[current_index..])?;

            decodeds.push(Decoded::ByteString(string));
            current_index += end_index;
        } else if current_value == b'i' {
            let (number, end_index) = parse_integer(&input[current_index..])?;

            decodeds.push(Decoded::Integer(number));
            current_index += end_index;
        } else if current_value == b'l' {
            let (inner, end_index) = parse_internal(&input[(current_index + 1)..])?;

            decodeds.push(Decoded::List(inner));
            current_index += end_index;
        } else if current_value == b'd' {
            let (inner, end_index) = parse_internal(&input[(current_index + 1)..])?;

            let mut key = None;
            let mut dict = HashMap::new();

            for item in inner {
                if let Some(k) = key {
                    let k = match k {
                        Decoded::ByteString(s) => Ok(s),
                        _ => Err(anyhow!("Unexpected type for key in dictionary")),
                    }?;
                    dict.insert(str::from_utf8(k)?, item);
                    key = None;
                } else {
                    key = Some(item);
                }
            }

            decodeds.push(Decoded::Dictionary(dict));
            current_index += end_index;
        } else if current_value == b'e' {
            break;
        } else {
            return Err(anyhow!(
                "Unexpected value \"{current_value}\" at {current_index}"
            ));
        }
    }

    Ok((decodeds, current_index + 1))
}

#[derive(PartialEq, Debug, Clone)]
pub enum Decoded<'a> {
    ByteString(&'a [u8]),
    Integer(i64),
    List(Vec<Decoded<'a>>),
    Dictionary(HashMap<&'a str, Decoded<'a>>),
}

impl<'a> Display for Decoded<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decoded::ByteString(s) => write!(f, "\"{s:?}\""),
            Decoded::Integer(i) => write!(f, "{i}"),
            Decoded::List(l) => {
                write!(f, "[")?;

                for item in l {
                    write!(f, "{item},")?;
                }

                write!(f, "]")
            }
            Decoded::Dictionary(d) => {
                write!(f, "{{")?;

                for (key, value) in d {
                    write!(f, "\t{key}: {value},")?;
                }

                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_string() {
        let input = "5:hello";
        let expected = vec![Decoded::ByteString(b"hello")];

        assert_eq!(expected, decode_bencoded(input.as_bytes()).unwrap());
    }

    #[test]
    fn parses_integer() {
        let input = "i52e";
        let expected = vec![Decoded::Integer(52)];

        assert_eq!(expected, decode_bencoded(input.as_bytes()).unwrap());
    }

    #[test]
    fn parses_multiple_values() {
        let input = "5:helloi52e";
        let expected = vec![Decoded::ByteString(b"hello"), Decoded::Integer(52)];

        assert_eq!(expected, decode_bencoded(input.as_bytes()).unwrap());
    }

    #[test]
    fn parses_negative_integer() {
        let input = "i-52e";
        let expected = vec![Decoded::Integer(-52)];

        assert_eq!(expected, decode_bencoded(input.as_bytes()).unwrap());
    }

    #[test]
    fn parses_list() {
        let input = "l4:speni7ed4:spami7eee";
        let expected = vec![Decoded::List(vec![
            Decoded::ByteString(b"spen"),
            Decoded::Integer(7),
            Decoded::Dictionary(HashMap::from([("spam", Decoded::Integer(7))])),
        ])];

        assert_eq!(expected, decode_bencoded(input.as_bytes()).unwrap());
    }

    #[test]
    fn parses_dict() {
        let input = "d4:spami7e3:keyd4:spami7eee";
        let expected = vec![Decoded::Dictionary(HashMap::from([
            ("spam", Decoded::Integer(7)),
            (
                "key",
                Decoded::Dictionary(HashMap::from([("spam", Decoded::Integer(7))])),
            ),
        ]))];

        assert_eq!(expected, decode_bencoded(input.as_bytes()).unwrap());
    }
}
