use crate::error::TaggerError;
use regex::Regex;
use std::collections::HashMap;

pub enum Searcher {
    And {
        lhs: Box<Searcher>,
        rhs: Box<Searcher>,
    },
    Or {
        lhs: Box<Searcher>,
        rhs: Box<Searcher>,
    },
    Not {
        lhs: Box<Searcher>,
    },
    Tag {
        regex: Regex,
    },
    Equal {
        tag_regex: Regex,
        value_regex: Regex,
    },
    Less {
        tag_regex: Regex,
        rhs: i32,
    },
    LessEqual {
        tag_regex: Regex,
        rhs: i32,
    },
    Greater {
        tag_regex: Regex,
        rhs: i32,
    },
    GreaterEqual {
        tag_regex: Regex,
        rhs: i32,
    },
}

impl Searcher {
    pub fn new_and(lhs: Searcher, rhs: Searcher) -> Self {
        Searcher::And {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn new_or(lhs: Searcher, rhs: Searcher) -> Self {
        Searcher::Or {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn new_not(lhs: Searcher) -> Self {
        Searcher::Not { lhs: Box::new(lhs) }
    }

    pub fn new_tag(regex: &str) -> Result<Self, TaggerError> {
        let regex = Regex::new(&expand_regex(regex)).map_err(|err| TaggerError::Regex(err))?;
        Ok(Searcher::Tag { regex })
    }

    pub fn new_equal(tag_regex: &str, value_regex: &str) -> Result<Self, TaggerError> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| TaggerError::Regex(err))?;
        let value_regex =
            Regex::new(&expand_regex(value_regex)).map_err(|err| TaggerError::Regex(err))?;
        Ok(Searcher::Equal {
            tag_regex,
            value_regex,
        })
    }

    pub fn new_inequal(tag_regex: &str, value_regex: &str) -> Result<Self, TaggerError> {
        let equal = Searcher::new_equal(tag_regex, value_regex)?;
        Ok(Searcher::new_not(equal))
    }

    pub fn new_less(tag_regex: &str, rhs: &str) -> Result<Self, TaggerError> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| TaggerError::Regex(err))?;
        let rhs = rhs
            .parse::<i32>()
            .map_err(|err| TaggerError::IntParse(err))?;
        Ok(Searcher::Less { tag_regex, rhs })
    }

    pub fn new_less_equal(tag_regex: &str, rhs: &str) -> Result<Self, TaggerError> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| TaggerError::Regex(err))?;
        let rhs = rhs
            .parse::<i32>()
            .map_err(|err| TaggerError::IntParse(err))?;
        Ok(Searcher::LessEqual { tag_regex, rhs })
    }

    pub fn new_greater(tag_regex: &str, rhs: &str) -> Result<Self, TaggerError> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| TaggerError::Regex(err))?;
        let rhs = rhs
            .parse::<i32>()
            .map_err(|err| TaggerError::IntParse(err))?;
        Ok(Searcher::Greater { tag_regex, rhs })
    }

    pub fn new_greater_equal(tag_regex: &str, rhs: &str) -> Result<Self, TaggerError> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| TaggerError::Regex(err))?;
        let rhs = rhs
            .parse::<i32>()
            .map_err(|err| TaggerError::IntParse(err))?;
        Ok(Searcher::GreaterEqual { tag_regex, rhs })
    }

    pub fn is_match(&self, tags: &HashMap<String, Option<String>>) -> bool {
        match self {
            Searcher::And { lhs, rhs } => {
                let l = lhs.is_match(tags);
                if l {
                    rhs.is_match(tags)
                } else {
                    // short-circuit
                    l
                }
            }
            Searcher::Or { lhs, rhs } => {
                let l = lhs.is_match(tags);
                if l {
                    // short-circuit
                    l
                } else {
                    rhs.is_match(tags)
                }
            }
            Searcher::Not { lhs } => !lhs.is_match(tags),
            Searcher::Tag { regex } => !get_values_by_tag_regex(tags, regex).is_empty(),
            Searcher::Equal {
                tag_regex,
                value_regex,
            } => check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                value_regex.is_match(tag_value)
            }),
            Searcher::Less { tag_regex, rhs } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value < *rhs;
                    }
                    false
                })
            }
            Searcher::LessEqual { tag_regex, rhs } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value <= *rhs;
                    }
                    false
                })
            }
            Searcher::Greater { tag_regex, rhs } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value > *rhs;
                    }
                    false
                })
            }
            Searcher::GreaterEqual { tag_regex, rhs } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value >= *rhs;
                    }
                    false
                })
            }
        }
    }
}

// Returnvalue references keys in @tags
fn get_values_by_tag_regex<'a>(
    tags: &'a HashMap<String, Option<String>>,
    tag_regex: &Regex,
) -> Vec<&'a Option<String>> {
    let mut result: Vec<&'a Option<String>> = Vec::new();
    for (tag, value) in tags {
        if tag_regex.is_match(tag) {
            result.push(value);
        }
    }
    result
}

// Returns true if one value of matching tags passes test
fn check_values_by_tag_regex<F>(
    tags: &HashMap<String, Option<String>>,
    tag_regex: &Regex,
    test: F,
) -> bool
where
    F: Fn(&str) -> bool,
{
    let values = get_values_by_tag_regex(tags, tag_regex);
    for value in values {
        match value {
            Some(tag_value) => {
                if test(tag_value) {
                    // short-circuit on match
                    return true;
                }
            }
            None => (), // This tag has no value, go on
        }
    }
    false
}

// Expand regex with anchors to match whole string
// TODO Should regex be put in non-capture-group (?: ) for safety?
pub fn expand_regex(regex: &str) -> String {
    format!("^{regex}$")
}
