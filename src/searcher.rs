use crate::error::{Result, XTagError};
use crate::XTags;
use regex::Regex;
use std::fmt;

/// Searcher variants.
pub enum Searcher {
    /// Logical and.
    And {
        lhs: Box<Searcher>,
        rhs: Box<Searcher>,
    },

    /// Logical or.
    Or {
        lhs: Box<Searcher>,
        rhs: Box<Searcher>,
    },

    /// Logical not.
    Not { lhs: Box<Searcher> },

    /// Matches tag.
    Tag { regex: Regex },

    /// Matches value.
    Equal {
        tag_regex: Regex,
        value_regex: Regex,
    },

    /// Matches if integer value is less than value.    
    Less { tag_regex: Regex, value: i32 },

    /// Matches if integer value is less or equal than rhs.
    LessEqual { tag_regex: Regex, value: i32 },

    /// Matches if integer value is greater than rhs.
    Greater { tag_regex: Regex, value: i32 },

    /// Matches if integer value is greater or equal than rhs.
    GreaterEqual { tag_regex: Regex, value: i32 },
}

impl Searcher {
    /// Returns new and Searcher.
    ///
    /// Matches when both elements match. Uses short-circuit evaluation. That means that when
    /// the left arm is already false, the right arm is not executed.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("foo".to_string(), None);
    /// tags.insert("bar".to_string(), None);
    /// let search = Searcher::new_and(Searcher::new_tag("foo").unwrap(), Searcher::new_tag("bar").unwrap());
    /// assert!(search.is_match(&tags) == true);
    /// ```
    pub fn new_and(lhs: Searcher, rhs: Searcher) -> Self {
        Searcher::And {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Returns new or Searcher.
    ///
    /// Matches when at least one element matches. Uses short-circuit evaluation. That means that when
    /// the left arm matches already, the right arm is not executed.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("foo".to_string(), None);
    /// let search = Searcher::new_or(Searcher::new_tag("foo").unwrap(), Searcher::new_tag("bar").unwrap());
    /// assert!(search.is_match(&tags) == true);
    /// ```
    pub fn new_or(lhs: Searcher, rhs: Searcher) -> Self {
        Searcher::Or {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Returns new not Searcher.
    ///
    /// Matches when the contained element doesn't match.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("bar".to_string(), None);
    /// let search = Searcher::new_not(Searcher::new_tag("foo").unwrap());
    /// assert!(search.is_match(&tags) == true);
    /// ```
    pub fn new_not(lhs: Searcher) -> Self {
        Searcher::Not { lhs: Box::new(lhs) }
    }

    /// Returns new tag Searcher.
    ///
    /// Matches when the regular expression matches. The expression is expanded with anchors to match
    /// the whole tag.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("foo".to_string(), None);
    /// let search = Searcher::new_tag("foo").unwrap();
    /// assert!(search.is_match(&tags) == true);
    /// ```
    ///
    /// # Errors
    ///
    /// - XTagError::Regex if the regex argument is not a valid regular expression
    pub fn new_tag(regex: &str) -> Result<Self> {
        let regex = Regex::new(&expand_regex(regex)).map_err(|err| XTagError::Regex(err))?;
        Ok(Searcher::Tag { regex })
    }

    /// Returns new equal Searcher.
    ///
    /// tag_regex specifies which tags are checked and value_regex is matched against the associated
    /// values. Matches when one value of one matching tag matches. The regular expressions are
    /// expanded with anchors to match the whole tag or value.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("bar".to_string(), Some("foo".to_string()));
    /// tags.insert("baz".to_string(), Some("qux".to_string()));
    /// let search = Searcher::new_equal("ba.", "qu.").unwrap();
    /// assert!(search.is_match(&tags) == true);
    /// ```
    ///
    /// # Errors
    ///
    /// - XTagError::Regex if tag_regex or value_regex are not a valid regular expression
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

    /// Returns Searcher for inequality
    ///
    /// Combines an equal and a not searcher to test for inequality
    ///
    /// # Errors
    ///
    /// - XTagError::Regex if tag_regex or value_regex are not a valid regular expression
    pub fn new_inequal(tag_regex: &str, value_regex: &str) -> Result<Self> {
        let equal = Searcher::new_equal(tag_regex, value_regex)?;
        Ok(Searcher::new_not(equal))
    }

    /// Returns new less Searcher.
    ///
    /// tag_regex specifies which tags are checked and rhs is matched against the integer
    /// representation of the associated values. Matches when one value of one matching tag matches.
    /// tag_regex is expanded with anchors to match the whole tag. If the value cannot be converted
    /// to integer that's no match.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("bar".to_string(), Some("10".to_string()));
    /// tags.insert("baz".to_string(), Some("100".to_string()));
    /// let search = Searcher::new_less("ba.", "50").unwrap();
    /// assert!(search.is_match(&tags) == true);
    /// ```
    ///
    /// # Errors
    ///
    /// - XTagError::Regex if tag_regex is not a valid regular expression
    /// - XtagError::IntParse if rhs can't be parsed into an integer
    pub fn new_less(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::Less { tag_regex, value })
    }

    /// Returns new less or equal Searcher.
    ///
    /// tag_regex specifies which tags are checked and rhs is matched against the integer
    /// representation of the associated values. Matches when one value of one matching tag matches.
    /// tag_regex is expanded with anchors to match the whole tag. If the value cannot be converted
    /// to integer that's no match.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("bar".to_string(), Some("10".to_string()));
    /// tags.insert("baz".to_string(), Some("100".to_string()));
    /// let search = Searcher::new_less_equal("ba.", "10").unwrap();
    /// assert!(search.is_match(&tags) == true);
    /// ```
    pub fn new_less_equal(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::LessEqual { tag_regex, value })
    }

    /// Returns new greater Searcher.
    ///
    /// tag_regex specifies which tags are checked and rhs is matched against the integer
    /// representation of the associated values. Matches when one value of one matching tag matches.
    /// tag_regex is expanded with anchors to match the whole tag. If the value cannot be converted
    /// to integer that's no match.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("bar".to_string(), Some("10".to_string()));
    /// tags.insert("baz".to_string(), Some("100".to_string()));
    /// let search = Searcher::new_greater("ba.", "50").unwrap();
    /// assert!(search.is_match(&tags) == true);
    /// ```
    pub fn new_greater(tag_regex: &str, value: &str) -> Result<Self> {
        let tag_regex =
            Regex::new(&expand_regex(tag_regex)).map_err(|err| XTagError::Regex(err))?;
        let value = value
            .parse::<i32>()
            .map_err(|err| XTagError::IntParse(err))?;
        Ok(Searcher::Greater { tag_regex, value })
    }

    /// Returns new greater or equal Searcher.
    ///
    /// tag_regex specifies which tags are checked and rhs is matched against the integer
    /// representation of the associated values. Matches when one value of one matching tag matches.
    /// tag_regex is expanded with anchors to match the whole tag. If the value cannot be converted
    /// to integer that's no match.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use xtag::Searcher;
    /// # use xtag::XTags;
    /// let mut tags: XTags = HashMap::new();
    /// tags.insert("bar".to_string(), Some("10".to_string()));
    /// tags.insert("baz".to_string(), Some("100".to_string()));
    /// let search = Searcher::new_greater_equal("ba.", "10").unwrap();
    /// assert!(search.is_match(&tags) == true);
    /// ```
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

impl fmt::Display for Searcher {
    /// Doesn't necessarily reproduce the exact term this Searcher resulted from.
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
