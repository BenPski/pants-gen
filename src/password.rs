use std::fmt::Display;
use std::hash::Hash;
use std::usize;
use std::{collections::HashSet, str::FromStr};

use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};
use thiserror::Error;

use crate::interval::Interval;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordSpec {
    length: usize,
    choices: Choices,
}

impl Default for PasswordSpec {
    fn default() -> Self {
        let mut choices = Choices::new();
        choices.push(CharStyle::Upper.at_least(1));
        choices.push(CharStyle::Lower.at_least(1));
        choices.push(CharStyle::Number.at_least(1));
        choices.push(CharStyle::Symbol.at_least(1));
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
}

// password spec specified as a string would look something like
// 16//1+|:upper://5-|:lower://2|Aa
// (Upper, at least 1) (Lower, at most 5) (Custom(Aa), exactly 2) length=16

impl FromStr for PasswordSpec {
    type Err = PasswordParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start();
        let first_sep = "//".to_string();
        let first_sep_char = first_sep.chars().last().unwrap();
        let second_sep = "|".to_string();
        let mut spec = PasswordSpec::new();
        let mut stack = String::new();
        let chars: Vec<char> = s.chars().collect();
        // parse length first
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            stack.push(c);
            i += 1;
            if stack.ends_with(&first_sep) {
                if let Ok(length) = stack[..stack.len() - first_sep.len()].parse() {
                    spec = spec.length(length);
                    stack = String::new();
                    break;
                } else {
                    return Err(PasswordParseError::InvalidLength(
                        stack[..stack.len() - first_sep.len()].to_string(),
                    ));
                }
            }
        }

        // parse choices
        let mut choice = (None, None);
        while i < chars.len() {
            let c = chars[i];
            if choice.0.is_none() {
                stack.push(c);
                if stack.ends_with(&second_sep) {
                    if let Ok(interval) =
                        stack[..stack.len() - second_sep.len()].parse::<Interval>()
                    {
                        choice.0 = Some(interval);
                        stack = String::new();
                    } else {
                        return Err(PasswordParseError::BadInterval(
                            stack[..stack.len() - second_sep.len()].to_string(),
                        ));
                    }
                }
                i += 1;
            } else if choice.1.is_none() {
                if c != first_sep_char && stack.ends_with(&first_sep) {
                    if let Ok(charset) = stack[..stack.len() - first_sep.len()].parse::<CharStyle>()
                    {
                        choice.1 = Some(charset);
                        stack = String::new();
                    } else {
                        return Err(PasswordParseError::BadCharset(
                            stack[..stack.len() - first_sep.len()].to_string(),
                        ));
                    }
                }

                stack.push(c);
                i += 1;
            } else {
                spec = spec.include(Choice::from_interval(choice.0.unwrap(), choice.1.unwrap()));
                choice = (None, None);
            }
        }

        // since parsing requires a peek, need to handle the very end of the string
        // having a trailing // is valid
        if stack.ends_with(&second_sep) {
            if let Ok(charset) = stack[..stack.len() - second_sep.len()].parse::<CharStyle>() {
                choice.1 = Some(charset);
                stack = String::new();
            } else {
                return Err(PasswordParseError::BadCharset(
                    stack[..stack.len() - second_sep.len()].to_string(),
                ));
            }
        }

        if !stack.is_empty() && choice.1.is_none() {
            if let Ok(charset) = stack.parse::<CharStyle>() {
                choice.1 = Some(charset);
            } else {
                return Err(PasswordParseError::BadCharset(
                    stack[..stack.len()].to_string(),
                ));
            }
        }

        match choice {
            (Some(interval), Some(charset)) => {
                spec = spec.include(Choice::from_interval(interval, charset));
            }
            (None, None) => {}
            _ => {
                return Err(PasswordParseError::ImproperFormat);
            }
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
            .push(Choice::from_interval(interval, CharStyle::Upper));
        self
    }
    pub fn upper_at_least(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Upper.at_least(size));
        self
    }
    pub fn upper_at_most(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Upper.at_most(size));
        self
    }
    pub fn upper_exactly(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Upper.exactly(size));
        self
    }
    pub fn lower(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, CharStyle::Lower));

        self
    }
    pub fn lower_at_least(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Lower.at_least(size));
        self
    }
    pub fn lower_at_most(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Lower.at_most(size));
        self
    }
    pub fn lower_exactly(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Lower.exactly(size));
        self
    }
    pub fn number(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, CharStyle::Number));

        self
    }
    pub fn number_at_least(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Number.at_least(size));
        self
    }
    pub fn number_at_most(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Number.at_most(size));
        self
    }
    pub fn number_exactly(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Number.exactly(size));
        self
    }
    pub fn symbol(mut self, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, CharStyle::Symbol));

        self
    }
    pub fn symbol_at_least(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Symbol.at_least(size));
        self
    }
    pub fn symbol_at_most(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Symbol.at_most(size));
        self
    }
    pub fn symbol_exactly(mut self, size: usize) -> Self {
        self.choices.push(CharStyle::Symbol.exactly(size));
        self
    }

    pub fn custom(mut self, chars: Vec<char>, interval: Interval) -> Self {
        self.choices
            .push(Choice::from_interval(interval, CharStyle::Custom(chars)));

        self
    }
    pub fn custom_at_least(mut self, chars: Vec<char>, size: usize) -> Self {
        self.choices.push(CharStyle::Custom(chars).at_least(size));
        self
    }
    pub fn custom_at_most(mut self, chars: Vec<char>, size: usize) -> Self {
        self.choices.push(CharStyle::Custom(chars).at_most(size));
        self
    }
    pub fn custom_exactly(mut self, chars: Vec<char>, size: usize) -> Self {
        self.choices.push(CharStyle::Custom(chars).exactly(size));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CharStyle {
    Upper,
    Lower,
    Number,
    Symbol,
    Custom(Vec<char>),
}

impl CharStyle {
    fn to_charset(&self) -> Vec<char> {
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

impl Display for CharStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharStyle::Upper => write!(f, ":upper:")?,
            CharStyle::Lower => write!(f, ":lower:")?,
            CharStyle::Number => write!(f, ":number:")?,
            CharStyle::Symbol => write!(f, ":symbol:")?,
            CharStyle::Custom(c) => write!(f, "{}", c.iter().collect::<String>())?,
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CharStyleParseError {
    #[error("No character set")]
    NoCharset,
    #[error("Specified a :pattern:, but it wasn't recognized")]
    UnrecognizedPattern,
}

impl FromStr for CharStyle {
    type Err = CharStyleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ":upper:" => Ok(CharStyle::Upper),
            ":lower:" => Ok(CharStyle::Lower),
            ":number:" => Ok(CharStyle::Number),
            ":symbol:" => Ok(CharStyle::Symbol),
            _ => {
                let chars = s.chars().collect::<Vec<_>>();
                if s.is_empty() {
                    Err(CharStyleParseError::NoCharset)
                } else if chars[0] == ':' && chars[s.len() - 1] == ':' {
                    Err(CharStyleParseError::UnrecognizedPattern)
                } else {
                    Ok(CharStyle::Custom(chars))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Choices {
    choices: HashSet<Choice>,
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

impl Choices {
    fn new() -> Self {
        Self {
            choices: HashSet::new(),
        }
    }

    fn push(&mut self, choice: Choice) {
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
    min: usize,
    max: usize,
    chars: CharStyle,
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
    #[error("Need both a character set and interval when specifying a choice, charset|interval")]
    NoInterval,
    #[error("Unable to parse the given interval")]
    BadInterval,
    #[error("Charset parse error, `{0}`")]
    CharStyle(CharStyleParseError),
}

// chars|interval -> Choice
impl FromStr for Choice {
    type Err = ChoiceParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s.rfind('|').ok_or(ChoiceParseError::NoInterval)?;
        let chars: CharStyle = s[..pos].parse().map_err(ChoiceParseError::CharStyle)?;
        let interval = s[pos + 1..]
            .parse()
            .map_err(|_| ChoiceParseError::BadInterval)?;
        Ok(Choice::from_interval(interval, chars))
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars)?;
        write!(f, "|")?;
        if self.min == self.max {
            write!(f, "{}", self.min)?;
        } else if self.min == usize::MIN {
            write!(f, "{}-", self.max)?;
        } else if self.max == usize::MAX {
            write!(f, "{}+", self.min)?;
        } else {
            write!(f, "{}-{}", self.min, self.max)?;
        }
        Ok(())
    }
}

impl Choice {
    fn new(min: usize, max: usize, chars: CharStyle) -> Option<Self> {
        if max >= min {
            Some(Self { min, max, chars })
        } else {
            None
        }
    }

    fn from_interval(interval: Interval, chars: CharStyle) -> Self {
        Self {
            min: interval.min,
            max: interval.max,
            chars,
        }
    }

    fn exactly(count: usize, chars: CharStyle) -> Self {
        Self {
            min: count,
            max: count,
            chars,
        }
    }

    fn at_least(count: usize, chars: CharStyle) -> Self {
        Self {
            min: count,
            max: usize::MAX,
            chars,
        }
    }

    fn at_most(count: usize, chars: CharStyle) -> Self {
        Self {
            min: usize::MIN,
            max: count,
            chars,
        }
    }

    fn active(&self) -> bool {
        self.max > 0
    }

    fn required(&self) -> bool {
        self.min > 0
    }

    fn get_required(&mut self) -> Vec<char> {
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
