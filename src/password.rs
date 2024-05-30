use std::collections::HashSet;
use std::hash::Hash;

use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};

pub struct Password {
    choices: Choices,
    length: usize,
}

impl Default for Password {
    fn default() -> Self {
        let mut choices = Choices::new();
        choices.push(CharStyle::Upper.at_least(1));
        choices.push(CharStyle::Lower.at_least(1));
        choices.push(CharStyle::Number.at_least(1));
        choices.push(CharStyle::Symbol.at_least(1));
        Password {
            choices,
            length: 32,
        }
    }
}

impl Password {
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
            min_length = min_length.checked_add(choice.min).unwrap_or(usize::MAX);
            max_length = max_length.checked_add(choice.max).unwrap_or(usize::MAX);
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

    pub fn upper(mut self, min: usize, max: usize) -> Self {
        if min <= max {
            self.choices.push(Choice {
                min,
                max,
                chars: CharStyle::Upper,
            })
        } else {
            self.choices.push(Choice {
                max,
                min,
                chars: CharStyle::Upper,
            })
        }
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
    pub fn lower(mut self, min: usize, max: usize) -> Self {
        if min <= max {
            self.choices.push(Choice {
                min,
                max,
                chars: CharStyle::Lower,
            })
        } else {
            self.choices.push(Choice {
                max,
                min,
                chars: CharStyle::Lower,
            })
        }
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
    pub fn number(mut self, min: usize, max: usize) -> Self {
        if min <= max {
            self.choices.push(Choice {
                min,
                max,
                chars: CharStyle::Number,
            })
        } else {
            self.choices.push(Choice {
                max,
                min,
                chars: CharStyle::Number,
            })
        }
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
    pub fn symbol(mut self, min: usize, max: usize) -> Self {
        if min <= max {
            self.choices.push(Choice {
                min,
                max,
                chars: CharStyle::Symbol,
            })
        } else {
            self.choices.push(Choice {
                max,
                min,
                chars: CharStyle::Symbol,
            })
        }
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

    pub fn custom(mut self, chars: Vec<char>, min: usize, max: usize) -> Self {
        if min <= max {
            self.choices.push(Choice {
                min,
                max,
                chars: CharStyle::Custom(chars),
            })
        } else {
            self.choices.push(Choice {
                max,
                min,
                chars: CharStyle::Custom(chars),
            })
        }
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

// TODO: generic character sets value
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
                // no real standard for allowed character sets for symbols, but I have some suspicions
                // about disallowed ones
                // for now not including quotes and backslash even though I think others could be
                // troublesome
                vec![
                    '!', '@', '#', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', '{', ']',
                    '}', '|', ':', ';', ',', '.', '?', '<', '>', '~',
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

    pub fn as_choice(self, min: usize, max: usize) -> Option<Choice> {
        Choice::new(min, max, self)
    }
}

#[derive(Debug, Clone)]
struct Choices {
    choices: HashSet<Choice>,
}

impl Choices {
    fn new() -> Self {
        Self {
            choices: HashSet::new(),
        }
    }

    fn push(&mut self, choice: Choice) {
        self.choices.insert(choice);
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

impl Choice {
    fn new(min: usize, max: usize, chars: CharStyle) -> Option<Self> {
        if max >= min {
            Some(Self { min, max, chars })
        } else {
            None
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
