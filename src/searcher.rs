use crate::error::{Result, XTagError};
use crate::XTags;
use regex::Regex;
use std::fmt;

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
        value: i32,
    },
    LessEqual {
        tag_regex: Regex,
        value: i32,
    },
    Greater {
        tag_regex: Regex,
        value: i32,
    },
    GreaterEqual {
        tag_regex: Regex,
        value: i32,
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

    pub fn new_tag(regex: &str) -> Result<Self> {
        let regex = Regex::new(&expand_regex(regex)).map_err(|err| XTagError::Regex(err))?;
        Ok(Searcher::Tag { regex })
    }

    pub fn new_equal(tag_regex: &str, value_regex: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value_regex =
            Regex::new(&expand_regex(value_regex)).map_err(|err| XTagError::Regex(err))?;
        Ok(Searcher::Equal {
            tag_regex,
            value_regex,
        })
    }

    pub fn new_inequal(tag_regex: &str, value_regex: &str) -> Result<Self> {
        let equal = Searcher::new_equal(tag_regex, value_regex)?;
        Ok(Searcher::new_not(equal))
    }

    pub fn new_less(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::Less { tag_regex, value })
    }

    pub fn new_less_equal(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::LessEqual { tag_regex, value })
    }

    pub fn new_greater(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::Greater { tag_regex, value })
    }

    pub fn new_greater_equal(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::GreaterEqual { tag_regex, value })
    }

    /// Evaluates Searcher against tags.
    pub fn is_match(&self, tags: &XTags) -> bool {
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
            Searcher::Less { tag_regex, value } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value < *value;
                    }
                    false
                })
            }
            Searcher::LessEqual { tag_regex, value } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value <= *value;
                    }
                    false
                })
            }
            Searcher::Greater { tag_regex, value } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value > *value;
                    }
                    false
                })
            }
            Searcher::GreaterEqual { tag_regex, value } => {
                check_values_by_tag_regex(tags, tag_regex, |tag_value: &str| -> bool {
                    if let Ok(tag_value) = tag_value.parse::<i32>() {
                        return tag_value >= *value;
                    }
                    false
                })
            }
        }
    }
}

/// Doesn't necessarily reproduce the exact term this Searcher resulted from.
impl fmt::Display for Searcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Searcher::And { lhs, rhs } => write!(f, "({}) AND ({})", lhs, rhs),
            Searcher::Or { lhs, rhs } => write!(f, "({}) OR ({})", lhs, rhs),
            Searcher::Not { lhs } => write!(f, "NOT ({})", lhs),
            Searcher::Tag { regex } => write!(f, "{}", regex),
            Searcher::Equal {
                tag_regex,
                value_regex,
            } => write!(f, "{} == {}", tag_regex, value_regex),
            Searcher::Less { tag_regex, value } => write!(f, "{} < {}", tag_regex, value),
            Searcher::LessEqual { tag_regex, value } => write!(f, "{} <= {}", tag_regex, value),
            Searcher::Greater { tag_regex, value } => write!(f, "{} > {}", tag_regex, value),
            Searcher::GreaterEqual { tag_regex, value } => write!(f, "{} >= {}", tag_regex, value),
        }
    }
}

// Returnvalue references keys in @tags
fn get_values_by_tag_regex<'a>(tags: &'a XTags, tag_regex: &Regex) -> Vec<&'a Option<String>> {
    let mut result: Vec<&'a Option<String>> = Vec::new();
    for (tag, value) in tags {
        if tag_regex.is_match(tag) {
            result.push(value);
        }
    }
    result
}

// Returns true if one value of matching tags passes test
fn check_values_by_tag_regex<F>(tags: &XTags, tag_regex: &Regex, test: F) -> bool
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

/// Expand regex with anchors to match whole string
///
/// Doesn't do anything if first and last characters are matching anchors.
// TODO Should regex be put in non-capture-group (?: ) for safety?
pub fn expand_regex(regex: &str) -> String {
    if regex.starts_with('^') && regex.ends_with('$') {
        regex.to_owned()
    } else {
        format!("^{regex}$")
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn test_stability(term: &str) {
        let term2 = format!("{}", compile_search(term).unwrap());
        let term3 = format!("{}", compile_search(&term2).unwrap());
        assert_eq!(term2, term3);
    }

    #[test]
    fn display_is_stable() {
        test_stability("a or b and c");
        test_stability("(a or b) and c");
        test_stability("a or (b and c)");
    }
}
