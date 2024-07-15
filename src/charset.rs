use std::{fmt::Display, str::FromStr};

use thiserror::Error;

use crate::{choice::Choice, interval::Interval};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Charset {
    Upper,
    Lower,
    Number,
    Symbol,
    Custom(Vec<char>),
}

impl Charset {
    pub fn to_charset(&self) -> Vec<char> {
        match self {
            Self::Upper => ('A'..='Z').collect(),
            Self::Lower => ('a'..='z').collect(),
            Self::Number => ('1'..='9').collect(),
            Self::Symbol => {
                // no real standard for allowed character sets for symbols
                // there are likely a few obvious ones that are concerns with escaping and are
                // interpretted as special characters at the command line that are removed
                vec![
                    '!', '@', '%', '^', '&', '*', '-', '_', '=', '+', ':', ';', ',', '.', '?', '~',
                ]
            }
            Self::Custom(v) => v.to_vec(),
        }
    }

    pub fn at_least(self, size: usize) -> Choice {
        Choice::at_least(size, self)
    }

    pub fn at_most(self, size: usize) -> Choice {
        Choice::at_most(size, self)
    }

    pub fn exactly(self, size: usize) -> Choice {
        Choice::exactly(size, self)
    }

    pub fn between(self, min: usize, max: usize) -> Option<Choice> {
        Choice::new(min, max, self)
    }

    pub fn from_interval(self, interval: Interval) -> Choice {
        Choice::from_interval(interval, self)
    }
}

impl Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Charset::Upper => write!(f, ":upper:")?,
            Charset::Lower => write!(f, ":lower:")?,
            Charset::Number => write!(f, ":number:")?,
            Charset::Symbol => write!(f, ":symbol:")?,
            Charset::Custom(c) => write!(f, "{}", c.iter().collect::<String>())?,
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CharsetParseError {
    #[error("No character set")]
    NoCharset,
    #[error("Specified a :pattern:, but `{0}` isn't recognized")]
    UnrecognizedPattern(String),
}

impl FromStr for Charset {
    type Err = CharsetParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ":upper:" => Ok(Charset::Upper),
            ":lower:" => Ok(Charset::Lower),
            ":number:" => Ok(Charset::Number),
            ":symbol:" => Ok(Charset::Symbol),
            _ => {
                let chars = s.chars().collect::<Vec<_>>();
                if s.is_empty() {
                    Err(CharsetParseError::NoCharset)
                } else if chars[0] == ':' && chars[s.len() - 1] == ':' {
                    Err(CharsetParseError::UnrecognizedPattern(s.to_string()))
                } else {
                    Ok(Charset::Custom(chars))
                }
            }
        }
    }
}
