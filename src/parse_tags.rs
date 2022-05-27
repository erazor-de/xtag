use std::collections::HashMap;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;

use crate::error::{Result, XTagError};
use crate::parser::Rule;
use crate::parser::SearchParser;
use crate::XTags;

fn eval_tag_with_value(pair: Pair<Rule>, container: &mut XTags) {
    let mut pairs = pair.into_inner();
    let tag = pairs.next().unwrap().as_str().to_string();
    let value = pairs.next().map(|v| v.as_str().to_string());
    container.insert(tag, value);
}

fn eval_tags(pairs: &mut Pairs<Rule>, container: &mut XTags) -> Result<()> {
    while pairs.peek().is_some() {
        let thing = pairs.next().unwrap();
        match thing.as_rule() {
            Rule::tag_with_value => eval_tag_with_value(thing, container),
            Rule::EOI => (),
            other => {
                return Err(XTagError::ParserImplementation(format!(
                    "unexpected rule {:?}",
                    other
                )));
            }
        }
    }
    Ok(())
}

/// Convert comma separated list of tag=value pairs to map
pub fn csl_to_map(string: &str) -> Result<XTags> {
    let mut result: XTags = HashMap::new();

    // pairs = Array of tag_with_value with final EOI
    let mut pairs = SearchParser::parse(Rule::comma_separated_tags_with_values, string)
        .map_err(|err| XTagError::Parser(err))?;
    eval_tags(&mut pairs, &mut result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::csl_to_map;

    #[test]
    fn grammar_tags_support_spaces() {
        let result = std::panic::catch_unwind(|| csl_to_map("a , b = c"));
        assert!(result.is_ok());
    }
}
