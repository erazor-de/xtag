use crate::error::TaggerError;
use crate::parser::Rule;
use crate::parser::SearchParser;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use std::collections::HashMap;

fn eval_tag_with_value(pair: Pair<Rule>, container: &mut HashMap<String, Option<String>>) {
    let mut pairs = pair.into_inner();
    let tag = pairs.next().unwrap().as_str().to_string();
    let value = pairs.next().map(|v| v.as_str().to_string());
    container.insert(tag, value);
}

fn eval_tags(
    pairs: &mut Pairs<Rule>,
    container: &mut HashMap<String, Option<String>>,
) -> Result<(), TaggerError> {
    while pairs.peek().is_some() {
        let thing = pairs.next().unwrap();
        match thing.as_rule() {
            Rule::tag_with_value => eval_tag_with_value(thing, container),
            Rule::EOI => (),
            other => {
                return Err(TaggerError::ParserImplementation(format!(
                    "unexpected rule {:?}",
                    other
                )));
            }
        }
    }
    Ok(())
}

// converts comma separated list to HashMap
pub fn parse_tags(string: &str) -> Result<HashMap<String, Option<String>>, TaggerError> {
    let mut result: HashMap<String, Option<String>> = HashMap::new();

    // pairs = Array of tag_with_value with final EOI
    let mut pairs = SearchParser::parse(Rule::comma_separated_tags_with_values, string)
        .map_err(|err| TaggerError::Parser(err))?;
    eval_tags(&mut pairs, &mut result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::parse_tags;

    #[test]
    fn grammar_tags_support_spaces() {
        let result = std::panic::catch_unwind(|| parse_tags("a , b = c"));
        assert!(result.is_ok());
    }
}
