use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub min: usize,
    pub max: usize,
}

impl Interval {
    pub fn new(min: usize, max: usize) -> Option<Self> {
        if min <= max {
            Some(Self { min, max })
        } else {
            None
        }
    }

    pub fn at_least(size: usize) -> Self {
        Self {
            min: size,
            max: usize::MAX,
        }
    }

    pub fn at_most(size: usize) -> Self {
        Self {
            min: usize::MIN,
            max: size,
        }
    }

    pub fn exactly(size: usize) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    pub fn safe(a: usize, b: usize) -> Self {
        if a <= b {
            Self { min: a, max: b }
        } else {
            Self { min: b, max: a }
        }
    }
}

#[derive(Debug, Error)]
pub enum IntervalParseError {
    #[error("Expect the interval to have the first value <= the second, got {0} <= {1}")]
    BadBounds(usize, usize),
    #[error("got `{0}`, expect the format for an interval to be: N, N+, N-, or A-B")]
    ImproperFormat(String),
}

impl FromStr for Interval {
    type Err = IntervalParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum Style {
            Exact,
            AtLeast,
            AtMost,
            Range,
        }
        let s = s.trim();
        let mut first = vec![];
        let mut second = vec![];
        let mut style = Style::Exact;
        for (i, c) in s.chars().enumerate() {
            if matches!(style, Style::Exact) {
                if c.is_numeric() {
                    first.push(c);
                } else if c == '+' && i + 1 == s.len() {
                    style = Style::AtLeast;
                } else if c == '-' && i + 1 == s.len() {
                    style = Style::AtMost;
                } else if c == '-' {
                    style = Style::Range;
                } else {
                    return Err(IntervalParseError::ImproperFormat(s.to_string()));
                }
            } else if matches!(style, Style::Range) {
                if c.is_numeric() {
                    second.push(c);
                } else {
                    return Err(IntervalParseError::ImproperFormat(s.to_string()));
                }
            }
        }
        match style {
            Style::Exact => {
                let n = first
                    .into_iter()
                    .collect::<String>()
                    .parse()
                    .map_err(|_| IntervalParseError::ImproperFormat(s.to_string()))?;
                Ok(Interval { min: n, max: n })
            }
            Style::AtLeast => {
                let n = first
                    .into_iter()
                    .collect::<String>()
                    .parse()
                    .map_err(|_| IntervalParseError::ImproperFormat(s.to_string()))?;
                Ok(Interval {
                    min: n,
                    max: usize::MAX,
                })
            }
            Style::AtMost => {
                let n = first
                    .into_iter()
                    .collect::<String>()
                    .parse()
                    .map_err(|_| IntervalParseError::ImproperFormat(s.to_string()))?;
                Ok(Interval {
                    min: usize::MIN,
                    max: n,
                })
            }
            Style::Range => {
                let a = first
                    .into_iter()
                    .collect::<String>()
                    .parse()
                    .map_err(|_| IntervalParseError::ImproperFormat(s.to_string()))?;
                let b = second
                    .into_iter()
                    .collect::<String>()
                    .parse()
                    .map_err(|_| IntervalParseError::ImproperFormat(s.to_string()))?;
                if a <= b {
                    Ok(Interval { min: a, max: b })
                } else {
                    Err(IntervalParseError::BadBounds(a, b))
                }
            }
        }
    }
}
