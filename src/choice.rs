use rand::prelude::SliceRandom;
use std::hash::Hash;
use std::{collections::HashSet, fmt::Display, str::FromStr};

use rand::thread_rng;
use thiserror::Error;

use crate::{
    charset::{Charset, CharsetParseError},
    interval::{Interval, IntervalParseError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choices {
    pub choices: HashSet<Choice>,
}

impl From<Vec<Choice>> for Choices {
    fn from(value: Vec<Choice>) -> Self {
        Choices {
            choices: HashSet::from_iter(value),
        }
    }
}

impl Display for Choices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for choice in &self.choices {
            write!(f, "//{}", choice)?;
        }
        Ok(())
    }
}

// Implementing FromStr needs an overall better parsing strategy
// #[derive(Debug, Error)]
// enum ChoicesParseError {
//
// }
//
// impl FromStr for Choices {
//     type Err = ChoicesParseError;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//
//     }
// }

impl Default for Choices {
    fn default() -> Self {
        Self::new()
    }
}

impl Choices {
    pub fn new() -> Self {
        Self {
            choices: HashSet::new(),
        }
    }

    pub fn push(&mut self, choice: Choice) {
        self.choices.replace(choice);
    }
}

impl IntoIterator for Choices {
    type Item = Choice;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.choices.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub(crate) min: usize,
    pub(crate) max: usize,
    pub chars: Charset,
}

// don't care about min and max count only care about the character sets being chosen from
impl PartialEq for Choice {
    fn eq(&self, other: &Self) -> bool {
        self.chars == other.chars
    }
}
impl Eq for Choice {}

impl Hash for Choice {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.chars.hash(state)
    }
}

#[derive(Debug, Error)]
pub enum ChoiceParseError {
    #[error("Unable to parse `{0}`, expect a form like interval|charset")]
    BadFormat(String),
    #[error("{0}")]
    BadInterval(IntervalParseError),
    #[error("{0}")]
    Charset(CharsetParseError),
}

// interval|charset -> Choice
impl FromStr for Choice {
    type Err = ChoiceParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s
            .find('|')
            .ok_or_else(|| ChoiceParseError::BadFormat(s.to_string()))?;
        let interval = s[..pos].parse().map_err(ChoiceParseError::BadInterval)?;
        let chars: Charset = s[pos + 1..].parse().map_err(ChoiceParseError::Charset)?;
        Ok(Choice::from_interval(interval, chars))
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min == self.max {
            write!(f, "{}", self.min)?;
        } else if self.min == usize::MIN {
            write!(f, "{}-", self.max)?;
        } else if self.max == usize::MAX {
            write!(f, "{}+", self.min)?;
        } else {
            write!(f, "{}-{}", self.min, self.max)?;
        }
        write!(f, "|")?;
        write!(f, "{}", self.chars)?;
        Ok(())
    }
}

impl Choice {
    pub fn new(min: usize, max: usize, chars: Charset) -> Option<Self> {
        if max >= min {
            Some(Self { min, max, chars })
        } else {
            None
        }
    }

    pub fn from_interval(interval: Interval, chars: Charset) -> Self {
        Self {
            min: interval.min,
            max: interval.max,
            chars,
        }
    }

    pub fn exactly(count: usize, chars: Charset) -> Self {
        Self {
            min: count,
            max: count,
            chars,
        }
    }

    pub fn at_least(count: usize, chars: Charset) -> Self {
        Self {
            min: count,
            max: usize::MAX,
            chars,
        }
    }

    pub fn at_most(count: usize, chars: Charset) -> Self {
        Self {
            min: usize::MIN,
            max: count,
            chars,
        }
    }

    pub(crate) fn active(&self) -> bool {
        self.max > 0
    }

    pub(crate) fn required(&self) -> bool {
        self.min > 0
    }

    pub(crate) fn get_required(&mut self) -> Vec<char> {
        let mut res = vec![];
        while self.required() {
            if let Some(c) = self.next() {
                res.push(c);
            }
        }
        res
    }
}

impl Iterator for Choice {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.active() {
            if self.min > 0 {
                self.min -= 1;
            }
            if self.max > 0 {
                self.max -= 1;
            }
            self.chars.to_charset().choose(&mut thread_rng()).copied()
        } else {
            None
        }
    }
}
