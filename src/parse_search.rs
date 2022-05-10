use crate::error::TaggerError;
use crate::parser::Rule;
use crate::parser::SearchParser;
use pest::iterators::Pair;
use pest::Parser;
use regex::Regex;
use std::collections::HashMap;

fn eval_binary_expr(
    pair: Pair<Rule>,
    tags: &HashMap<String, Option<String>>,
) -> Result<bool, TaggerError> {
    let mut pairs = pair.into_inner();
    let mut lhs = eval_expression(pairs.next().unwrap(), tags)?;
    while pairs.peek().is_some() {
        // (binary_op ~ unary_expr)*
        let operation = pairs.next().unwrap();
        let rhs = eval_expression(pairs.next().unwrap(), tags)?;
        match operation.as_rule() {
            Rule::and => lhs = lhs && rhs,
            Rule::or => lhs = lhs || rhs,
            op => {
                return Err(TaggerError::ParserImplementation(format!(
                    "unsupported binary operation {op:?}"
                )))
            }
        };
    }
    Ok(lhs)
}

// Expand regex with anchors to match whole string
// Should regex be put in non-capture-group (?: ) for safety?
fn expand_regex(regex: &str) -> String {
    format!("^{regex}$")
}

// Returnvalue references keys in @tags
fn get_values_by_tag_regex<'a>(
    tags: &'a HashMap<String, Option<String>>,
    tag_regex: &str,
) -> Result<Vec<&'a Option<String>>, TaggerError> {
    let mut result: Vec<&'a Option<String>> = Vec::new();
    let tag_re = Regex::new(&expand_regex(tag_regex)).map_err(|err| TaggerError::Regex(err))?;
    for (tag, value) in tags {
        if tag_re.is_match(tag) {
            result.push(value);
        }
    }
    Ok(result)
}

fn eval_tag(pair: Pair<Rule>, tags: &HashMap<String, Option<String>>) -> Result<bool, TaggerError> {
    let tag_regex = pair.as_str();
    let matches = get_values_by_tag_regex(tags, tag_regex)?;
    Ok(!matches.is_empty())
}

fn eval_unary_expr(
    pair: Pair<Rule>,
    tags: &HashMap<String, Option<String>>,
) -> Result<bool, TaggerError> {
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    if pairs.peek().is_some() {
        // unary_op ~ unary_expr
        let operation = first;
        let rhs = pairs.next().unwrap();
        match operation.as_rule() {
            Rule::not => Ok(!eval_expression(rhs, tags)?),
            op => Err(TaggerError::ParserImplementation(format!(
                "unsupported unary operation {op:?}"
            ))),
        }
    } else {
        // comparison
        eval_expression(first, tags)
    }
}

fn compare_inequality<T: std::str::FromStr + std::cmp::PartialOrd>(
    rule: Rule,
    a: &str,
    b: &str,
) -> Result<bool, TaggerError> {
    if let Ok(a) = a.parse::<T>() {
        if let Ok(b) = b.parse::<T>() {
            return match rule {
                Rule::less => Ok(a < b),
                Rule::less_equal => Ok(a <= b),
                Rule::greater => Ok(a > b),
                Rule::greater_equal => Ok(a >= b),
                op => Err(TaggerError::ParserImplementation(format!(
                    "unsupported comparison operation {op:?}"
                ))),
            };
        }
    }
    Ok(false)
}

// Equality is tested as regex, inequality operators are done after conversion
// to int
fn eval_comparison(
    pair: Pair<Rule>,
    tags: &HashMap<String, Option<String>>,
) -> Result<bool, TaggerError> {
    let mut pairs = pair.into_inner();
    let lhs = pairs.next().unwrap();
    if pairs.peek().is_some() {
        // tag ~ comparison_op ~ value
        let tag_regex = lhs.as_str();
        let operation = pairs.next().unwrap();
        let search_value = pairs.next().unwrap().as_str();
        let value_re =
            Regex::new(&expand_regex(search_value)).map_err(|err| TaggerError::Regex(err))?;

        // iterating values of all tags that match tag_regex
        for value in get_values_by_tag_regex(tags, tag_regex)? {
            match value {
                Some(tag_value) => match operation.as_rule() {
                    Rule::equal => {
                        // equality treats search_value as regex
                        if value_re.is_match(tag_value) {
                            // a value of some tag matches
                            return Ok(true);
                        }
                    }
                    // inequality treats search_value as integer
                    op => {
                        if compare_inequality::<i32>(op, tag_value, search_value)? {
                            // a value of some tag matches
                            return Ok(true);
                        }
                    }
                },
                None => (), // This tag has no value, go on
            }
        }
        Ok(false) // no value of any matching tag matched
    } else {
        // primary
        eval_expression(lhs, tags)
    }
}

fn eval_expression(
    pair: Pair<Rule>,
    tags: &HashMap<String, Option<String>>,
) -> Result<bool, TaggerError> {
    match pair.as_rule() {
        Rule::tag_with_regex => eval_tag(pair, tags),
        Rule::binary_expr => eval_binary_expr(pair, tags),
        Rule::unary_expr => eval_unary_expr(pair, tags),
        Rule::comparison => eval_comparison(pair, tags),
        rule => Err(TaggerError::ParserImplementation(format!(
            "unexpected grammar rule {rule:?}"
        ))),
    }
}

pub fn parse_search(
    term: &str,
    tags: &HashMap<String, Option<String>>,
) -> Result<bool, TaggerError> {
    // parse returns array of one rule + EOI. Start with first element here
    let pair = SearchParser::parse(Rule::search, term)
        .map_err(|err| TaggerError::Parser(err))?
        .next()
        .unwrap();
    eval_expression(pair, tags)
}

#[cfg(test)]
mod tests {
    use super::parse_search;
    use crate::parse_tags::parse_tags;

    fn find_in_string(term: &str, string: &str) -> bool {
        let map = parse_tags(string).unwrap();
        parse_search(term, &map).unwrap()
    }

    #[test]
    fn grammar_binary_operations_have_equal_priority_and_are_left_associative() {
        assert!(find_in_string("a AND b OR c AND d", "a,b,d") == true);
        assert!(find_in_string("a AND b OR c AND d", "c,d") == true);
        assert!(find_in_string("a AND b OR c AND d", "a,b") == false);

        assert!(find_in_string("a AND b OR c", "a,b") == true);
        assert!(find_in_string("a AND b OR c", "c") == true);
        assert!(find_in_string("a AND b OR c", "a") == false);

        assert!(find_in_string("(a AND b) OR c", "a,b") == true);
        assert!(find_in_string("(a AND b) OR c", "c") == true);

        assert!(find_in_string("a AND (b OR c)", "a,b") == true);
        assert!(find_in_string("a AND (b OR c)", "a,c") == true);
        assert!(find_in_string("a AND (b OR c)", "a") == false);

        assert!(find_in_string("NOT a AND b", "a,b") == false);
        assert!(find_in_string("NOT a AND b", "b") == true);
        assert!(find_in_string("NOT a AND b", "c") == false);

        assert!(find_in_string("NOT (a AND b)", "a,b") == false);
        assert!(find_in_string("NOT (a AND b)", "b") == true);
        assert!(find_in_string("NOT (a AND b)", "c") == true);
    }

    #[test]
    fn grammar_operator_notations_can_be_mixed() {
        assert!(find_in_string("a && b || c AND d", "c,d") == true);
        assert!(find_in_string("!a AND b", "b") == true);
    }

    #[test]
    fn grammar_operator_supports_variable_spaces() {
        assert!(find_in_string("(aANDb)ORc", "a,b") == false);
        assert!(find_in_string("(a AND b)OR c", "a,b") == true);
        assert!(find_in_string(" ( a and b ) or c ", "a,b") == true);
        assert!(find_in_string("(a&&b)||c", "a,b") == true)
    }

    #[test]
    fn grammar_supports_string_value_equality() {
        assert!(find_in_string("a == b AND c", "a=b,c") == true);
        assert!(find_in_string("a == b", "a=c") == false);
        assert!(find_in_string("a == 1 or a == 2", "a=1") == true);
        assert!(find_in_string("a == 1 or a == 2", "a=3") == false);
        assert!(find_in_string("a and b == 1 or b == 2", "a,b=1") == true);
        assert!(find_in_string("a and b == 1 or b == 2", "a,b=2") == true);
        assert!(find_in_string("a and b == 1 or b == 2", "a,b=3") == false);
        assert!(find_in_string("a and b == 1 or b == 2", "b=1") == false);
        assert!(find_in_string("a and b == 1 or b == 2", "a") == false);
    }

    #[test]
    fn grammar_supports_int_value_inequalities() {
        assert!(find_in_string("a > 1 AND a < 3", "a=2") == true);
        assert!(find_in_string("a > 1 AND a < 3", "a=1") == false);
        assert!(find_in_string("a > 1 AND a < 3", "a=3") == false);
        assert!(find_in_string("a > 1 AND a < 3", "a") == false);
    }

    #[test]
    fn grammar_panics_on_invalid_expression() {
        let result = std::panic::catch_unwind(|| find_in_string("a b c", "a,b,c"));
        assert!(result.is_err());
    }

    #[test]
    fn grammar_supports_regex_for_tags() {
        assert!(find_in_string("a+b", "aaab") == true);
        assert!(find_in_string("a+b", "aaabb") == false);
        assert!(find_in_string("a+b == c", "aaab=c") == true);
        assert!(find_in_string("a+b == c", "aaabb=c") == false);
    }

    #[test]
    fn grammar_supports_regex_for_values() {
        assert!(find_in_string("a==b+c", "a=bbbc") == true);
        assert!(find_in_string("a==b+c", "a=c") == false);
        assert!(find_in_string("a==b+c", "d=bbbc") == false);
    }

    #[test]
    fn grammar_supports_regex_groups_for_values() {
        assert!(find_in_string("a==(ab|cd)+e", "a=ababe") == true);
        assert!(find_in_string("a==f(ab|cd)+e", "a=fabcdcdabe") == true);
        assert!(find_in_string("a==(ab)+e", "a=e") == false);
    }

    #[test]
    fn grammar_supports_regex_groups_for_tags() {
        assert!(find_in_string("f(ab|cd)e==b+c", "fabe=bbbc") == true);
        assert!(find_in_string("(ab|cd)==b+c", "cd=bbbc") == true);
        assert!(find_in_string("(ab|cd)==b+c", "ac=bbbc") == false);
    }

    #[test]
    fn grammar_supports_all_in_one() {
        assert!(
            find_in_string(
                "f(ab|cd).*e == b[ac]d && g[^h] < 20 AND !i",
                "fabxe=bad,gj=10"
            ) == true
        );
        assert!(
            find_in_string(
                "f(ab|cd).*e == b[ac]d && g[^h] < 20 AND !i",
                "fabxe=bad,gj=10,i"
            ) == false
        );
    }
}
