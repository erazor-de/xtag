use crate::error::TaggerError;
use crate::parser::Rule;
use crate::parser::SearchParser;
use crate::searcher::Searcher;
use pest::iterators::Pair;
use pest::Parser;

fn eval_binary_expr(pair: Pair<Rule>) -> Result<Searcher, TaggerError> {
    let mut pairs = pair.into_inner();
    let mut lhs = eval_expression(pairs.next().unwrap())?;
    while pairs.peek().is_some() {
        // (binary_op ~ unary_expr)*
        let operation = pairs.next().unwrap();
        let rhs = eval_expression(pairs.next().unwrap())?;
        match operation.as_rule() {
            Rule::and => lhs = Searcher::new_and(lhs, rhs),
            Rule::or => lhs = Searcher::new_or(lhs, rhs),
            op => {
                return Err(TaggerError::ParserImplementation(format!(
                    "unsupported binary operation {op:?}"
                )))
            }
        };
    }
    Ok(lhs)
}

fn eval_tag(pair: Pair<Rule>) -> Result<Searcher, TaggerError> {
    let tag_regex = pair.as_str();
    Searcher::new_tag(tag_regex)
}

fn eval_unary_expr(pair: Pair<Rule>) -> Result<Searcher, TaggerError> {
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    if pairs.peek().is_some() {
        // unary_op ~ unary_expr
        let operation = first;
        let rhs = eval_expression(pairs.next().unwrap())?;
        match operation.as_rule() {
            Rule::not => Ok(Searcher::new_not(rhs)),
            op => Err(TaggerError::ParserImplementation(format!(
                "unsupported unary operation {op:?}"
            ))),
        }
    } else {
        // comparison
        eval_expression(first)
    }
}

// Equality is tested as regex, inequality operators are done after conversion
// to int
fn eval_comparison(pair: Pair<Rule>) -> Result<Searcher, TaggerError> {
    let mut pairs = pair.into_inner();
    let lhs = pairs.next().unwrap();
    if pairs.peek().is_some() {
        // tag ~ comparison_op ~ value
        let tag_regex = lhs.as_str();
        let operation = pairs.next().unwrap();
        let value = pairs.next().unwrap().as_str();

        match operation.as_rule() {
            Rule::equal => Searcher::new_equal(tag_regex, value),
            Rule::inequal => Searcher::new_inequal(tag_regex, value),
            Rule::less => Searcher::new_less(tag_regex, value),
            Rule::less_equal => Searcher::new_less_equal(tag_regex, value),
            Rule::greater => Searcher::new_greater(tag_regex, value),
            Rule::greater_equal => Searcher::new_greater_equal(tag_regex, value),
            op => Err(TaggerError::ParserImplementation(format!(
                "unsupported comparison operation {op:?}"
            ))),
        }
    } else {
        // primary
        eval_expression(lhs)
    }
}

fn eval_expression(pair: Pair<Rule>) -> Result<Searcher, TaggerError> {
    match pair.as_rule() {
        Rule::tag_with_regex => eval_tag(pair),
        Rule::binary_expr => eval_binary_expr(pair),
        Rule::unary_expr => eval_unary_expr(pair),
        Rule::comparison => eval_comparison(pair),
        rule => Err(TaggerError::ParserImplementation(format!(
            "unexpected grammar rule {rule:?}"
        ))),
    }
}

pub fn compile_search(term: &str) -> Result<Searcher, TaggerError> {
    // parse returns array of one rule + EOI. Start with first element here
    let pair = SearchParser::parse(Rule::search, term)
        .map_err(|err| TaggerError::Parser(err))?
        .next()
        .unwrap();
    eval_expression(pair)
}

#[cfg(test)]
mod tests {
    use super::compile_search;
    use crate::parse_tags::csl_to_map;

    fn find_in_string(term: &str, string: &str) -> bool {
        let tags = csl_to_map(string).unwrap();
        let searcher = compile_search(term).unwrap();
        searcher.is_match(&tags)
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
    fn grammar_tag_matches_are_case_sensitive() {
        assert!(find_in_string("a && !A", "a") == true);
        assert!(find_in_string("a && !A", "a,A") == false);
        assert!(find_in_string("!A", "a") == true);
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
    fn grammar_supports_string_value_inequality() {
        assert!(find_in_string(".* != b", "a=c,b=d") == true);
        assert!(find_in_string(".* != b", "a=b,b=d") == false);
    }

    #[test]
    fn grammar_supports_int_value_relations() {
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
