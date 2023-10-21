use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt::Display};

pub fn decode_bencoded(input: &str) -> Result<Vec<Decoded>> {
    let mut current_index = 0;
    let mut decodeds = vec![];

    while current_index < input.len() {
        if let Some(current_value) = input.chars().nth(current_index) {
            if current_value.is_ascii_digit() {
                let colon_index = input[current_index..].find(':').ok_or(anyhow!(
                    "Couldn't parse string start at {current_index} - missing colon"
                ))?;
                let number: usize = input[current_index..colon_index].parse()?;
                let end_index = colon_index + 1 + number;
                let string = &input[colon_index + 1..end_index];

                decodeds.push(Decoded::String(string));
                current_index = end_index;
            } else if current_value == 'i' {
                let end_index = input[current_index..].find('e').ok_or(anyhow!(
                    "Couldn't parse integer start at {current_index} - missing end (e)"
                ))?;

                let number = input[current_index..][1..end_index].parse()?;

                decodeds.push(Decoded::Integer(number));
                current_index += end_index + 1;
            } else if current_value == 'l' {
                let mut count_e = 1;
                let mut i = current_index + 1;

                while count_e > 0 {
                    let c = input
                        .chars()
                        .nth(i)
                        .ok_or(anyhow!("Couldn't find closing tag for list"))?;
                    match c {
                        'i' | 'l' | 'd' => {
                            count_e += 1;
                        }
                        'e' => {
                            count_e -= 1;
                        }
                        _ => {}
                    }
                    i += 1;
                }

                let inner_str = &input[(current_index + 1)..(i - 1)];

                decodeds.push(Decoded::List(decode_bencoded(inner_str)?));

                current_index = i + 1;
            } else if current_value == 'd' {
                todo!()
            } else {
                return Err(anyhow!(
                    "Unexpected value \"{current_value}\" at {current_index}"
                ));
            }
        }
    }

    Ok(decodeds)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Decoded<'a> {
    String(&'a str),
    Integer(i64),
    List(Vec<Decoded<'a>>),
    Dictionary(HashMap<&'a str, Decoded<'a>>),
}

impl<'a> Display for Decoded<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decoded::String(s) => write!(f, "\"{s}\""),
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
        let expected = vec![Decoded::String("hello")];

        assert_eq!(expected, decode_bencoded(input).unwrap());
    }

    #[test]
    fn parses_integer() {
        let input = "i52e";
        let expected = vec![Decoded::Integer(52)];

        assert_eq!(expected, decode_bencoded(input).unwrap());
    }

    #[test]
    fn parses_negative_integer() {
        let input = "i-52e";
        let expected = vec![Decoded::Integer(-52)];

        assert_eq!(expected, decode_bencoded(input).unwrap());
    }

    #[test]
    fn parses_list() {
        let input = "l4:speni7ee";
        let expected = vec![Decoded::List(vec![
            Decoded::String("spen"),
            Decoded::Integer(7),
        ])];

        assert_eq!(expected, decode_bencoded(input).unwrap());
    }

    #[test]
    fn parses_dict() {
        let input = "d4:spami7ee";
        let expected = vec![Decoded::Dictionary(HashMap::from([(
            "spam",
            Decoded::Integer(7),
        )]))];

        assert_eq!(expected, decode_bencoded(input).unwrap());
    }

    #[test]
    fn parses_nested_dict() {
        let input = "d4:spami7e3:keyd4:spami7eee";
        let expected = vec![Decoded::Dictionary(HashMap::from([
            ("spam", Decoded::Integer(7)),
            (
                "key",
                Decoded::Dictionary(HashMap::from([("spam", Decoded::Integer(7))])),
            ),
        ]))];

        assert_eq!(expected, decode_bencoded(input).unwrap());
    }
}
