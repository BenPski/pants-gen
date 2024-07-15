use std::fmt::Display;
use std::str::FromStr;

use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};
use thiserror::Error;

use crate::choice::{ChoiceParseError, Choices};
use crate::interval::Interval;
use crate::{charset::Charset, choice::Choice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordSpec {
    length: usize,
    choices: Choices,
}

impl Default for PasswordSpec {
    fn default() -> Self {
        let mut choices = Choices::new();
        choices.push(Charset::Upper.at_least(1));
        choices.push(Charset::Lower.at_least(1));
        choices.push(Charset::Number.at_least(1));
        choices.push(Charset::Symbol.at_least(1));
        PasswordSpec {
            choices,
            length: 32,
        }
    }
}

#[derive(Debug, Error)]
pub enum PasswordParseError {
    #[error("Password spec improperly formatted, expect something like length//interval|charset//interval|charset (likely an internal parsing error)")]
    ImproperFormat,
    #[error("Couldn't parse the length segment of the spec `{0}`, expects it to be the first segment of the spec (length//...).")]
    InvalidLength(String),
    #[error("Couldn't parse the interval `{0}`.")]
    BadInterval(String),
    #[error("Couldn't parse the charset `{0}`.")]
    BadCharset(String),
    #[error("{0}")]
    BadChoice(ChoiceParseError),
}

// password spec specified as a string would look something like
// 16//1+|:upper://5-|:lower://2|Aa
// (Upper, at least 1) (Lower, at most 5) (Custom(Aa), exactly 2) length=16

impl FromStr for PasswordSpec {
    type Err = PasswordParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start();
        let sep = "//".to_string();
        let sep_char = sep.chars().last().unwrap();
        // let second_sep = "|".to_string();
        let mut spec = PasswordSpec::new();
        let mut stack = String::new();
        let chars: Vec<char> = s.chars().collect();
        // parse length first
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            stack.push(c);
            i += 1;
            if stack.ends_with(&sep) {
                let length: usize = stack[..stack.len() - sep.len()].parse().map_err(|_| {
                    PasswordParseError::InvalidLength(stack[..stack.len() - sep.len()].to_string())
                })?;
                spec = spec.length(length);
                stack = String::new();
                break;
            }
        }

        // parse choices
        while i < chars.len() {
            let c = chars[i];
            if c != sep_char && stack.ends_with(&sep) {
                let choice = stack[..stack.len() - sep.len()]
                    .parse()
                    .map_err(PasswordParseError::BadChoice)?;
                spec = spec.include(choice);
                stack = String::new();
            }
            stack.push(c);
            i += 1;
        }

        // since parsing requires a peek, need to handle the very end of the string
        // having a trailing // is valid
        if stack.ends_with(&sep) {
            let choice = stack[..stack.len() - sep.len()]
                .parse()
                .map_err(PasswordParseError::BadChoice)?;
            spec = spec.include(choice);
            stack = String::new();
        }

        if !stack.is_empty() {
            let choice = stack[..stack.len()]
                .parse()
                .map_err(PasswordParseError::BadChoice)?;
            spec = spec.include(choice);
            // stack = String::new();
        }

        Ok(spec)
    }
}

impl Display for PasswordSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.length)?;
        write!(f, "{}", self.choices)
    }
}

impl PasswordSpec {
    pub fn new() -> Self {
        Self {
            choices: Choices::new(),
            length: 32,
        }
    }
    pub fn generate(&self) -> Option<String> {
        if self.check() {
            let mut characters = vec![];
            let mut active = Choices::new();
            for mut choice in self.choices.clone() {
                characters.extend(choice.get_required());
                active.push(choice);
            }

            let remaining = self.length - characters.len();
            let mut active: Vec<_> = active.into_iter().filter(|x| x.active()).collect();

            for _ in 0..remaining {
                if let Some(index) = (0..active.len()).choose(&mut thread_rng()) {
                    let c = active[index].next().unwrap();
                    characters.push(c);
                    if !active[index].active() {
                        active.remove(index);
                    }
                }
            }

            characters.shuffle(&mut thread_rng());
            Some(characters.into_iter().collect())
        } else {
            None
        }
    }

    fn check(&self) -> bool {
        let mut min_length: usize = 0;
        let mut max_length: usize = 0;
        for choice in &self.choices.choices {
            min_length = min_length.saturating_add(choice.min);
            max_length = max_length.saturating_add(choice.max);
        }
        min_length <= self.length && self.length <= max_length
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn include(mut self, choice: Choice) -> Self {
        self.choices.push(choice);
        self
    }

    pub fn upper(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, Charset::Upper));
        self
    }
    pub fn upper_at_least(mut self, size: usize) -> Self {
        self.choices.push(Charset::Upper.at_least(size));
        self
    }
    pub fn upper_at_most(mut self, size: usize) -> Self {
        self.choices.push(Charset::Upper.at_most(size));
        self
    }
    pub fn upper_exactly(mut self, size: usize) -> Self {
        self.choices.push(Charset::Upper.exactly(size));
        self
    }
    pub fn lower(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, Charset::Lower));

        self
    }
    pub fn lower_at_least(mut self, size: usize) -> Self {
        self.choices.push(Charset::Lower.at_least(size));
        self
    }
    pub fn lower_at_most(mut self, size: usize) -> Self {
        self.choices.push(Charset::Lower.at_most(size));
        self
    }
    pub fn lower_exactly(mut self, size: usize) -> Self {
        self.choices.push(Charset::Lower.exactly(size));
        self
    }
    pub fn number(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, Charset::Number));

        self
    }
    pub fn number_at_least(mut self, size: usize) -> Self {
        self.choices.push(Charset::Number.at_least(size));
        self
    }
    pub fn number_at_most(mut self, size: usize) -> Self {
        self.choices.push(Charset::Number.at_most(size));
        self
    }
    pub fn number_exactly(mut self, size: usize) -> Self {
        self.choices.push(Charset::Number.exactly(size));
        self
    }
    pub fn symbol(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, Charset::Symbol));

        self
    }
    pub fn symbol_at_least(mut self, size: usize) -> Self {
        self.choices.push(Charset::Symbol.at_least(size));
        self
    }
    pub fn symbol_at_most(mut self, size: usize) -> Self {
        self.choices.push(Charset::Symbol.at_most(size));
        self
    }
    pub fn symbol_exactly(mut self, size: usize) -> Self {
        self.choices.push(Charset::Symbol.exactly(size));
        self
    }

    pub fn custom(mut self, chars: Vec<char>, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, Charset::Custom(chars)));

        self
    }
    pub fn custom_at_least(mut self, chars: Vec<char>, size: usize) -> Self {
        self.choices.push(Charset::Custom(chars).at_least(size));
        self
    }
    pub fn custom_at_most(mut self, chars: Vec<char>, size: usize) -> Self {
        self.choices.push(Charset::Custom(chars).at_most(size));
        self
    }
    pub fn custom_exactly(mut self, chars: Vec<char>, size: usize) -> Self {
        self.choices.push(Charset::Custom(chars).exactly(size));
        self
    }
}
