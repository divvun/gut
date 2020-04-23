use crate::github::RemoteRepo;
use regex::{Error as RegexError, Regex, RegexBuilder};
use std::path::PathBuf;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub struct Filter {
    regex: Regex,
}

impl FromStr for Filter {
    type Err = RegexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RegexBuilder::new(s)
            .case_insensitive(true)
            .build()
            .map(|regex| Filter { regex })
    }
}

impl Filter {
    pub fn is_match(&self, pattern: &str) -> bool {
        self.regex.is_match(pattern)
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.regex)
    }
}

pub trait Filterable {
    fn is_match(&self, filter: &Filter) -> bool;
    fn filter<T: Filterable>(vec: Vec<T>, filter: &Filter) -> Vec<T> {
        vec.into_iter().filter(|f| f.is_match(filter)).collect()
    }
    fn filter_with_option<T: Filterable>(vec: Vec<T>, option: &Option<Filter>) -> Vec<T> {
        match option {
            Some(regex) => <T as Filterable>::filter(vec, &regex),
            None => vec,
        }
    }
}

impl Filterable for RemoteRepo {
    fn is_match(&self, filter: &Filter) -> bool {
        filter.is_match(&self.name)
    }
}

impl Filterable for PathBuf {
    fn is_match(&self, filter: &Filter) -> bool {
        match self.to_str() {
            Some(v) => filter.is_match(v),
            None => false,
        }
    }
}
