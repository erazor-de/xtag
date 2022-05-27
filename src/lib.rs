mod bookmarks;
mod error;
mod parse_search;
mod parse_tags;
mod parser;
mod searcher;

use std::collections::HashMap;
use std::path::PathBuf;
use std::str;

use itertools::Itertools;
use regex::Regex;

pub use crate::bookmarks::get_bookmark;
pub use crate::error::{Result, XTagError};
pub use crate::parse_search::compile_search;
pub use crate::parse_tags::csl_to_map;
use crate::parser::Rule;
pub use crate::searcher::Searcher;

pub type XTags = HashMap<String, Option<String>>;

static XATTR_NAME: &'static str = "user.xtag";

/// Convert map to comma separated list of tag=value pairs
pub fn map_to_csl(set: &XTags) -> String {
    set.iter()
        .map(|(tag, value)| match value {
            Some(value) => tag.to_string() + "=" + value,
            None => tag.to_string(),
        })
        .join(",")
}

/// Get tags for file as map
pub fn get_tags(path: &PathBuf) -> Result<XTags> {
    let xattrs = xattr::get(path, XATTR_NAME).map_err(|err| XTagError::File(err))?;
    match xattrs {
        Some(value) => {
            let string = str::from_utf8(&value).map_err(|err| XTagError::Charset(err))?;
            csl_to_map(string)
        }
        None => csl_to_map(""),
    }
}

/// Set tags for file from map
///
/// The used utf-8 string format is architecture independent.
pub fn set_tags(path: &PathBuf, tags: &XTags) -> Result<()> {
    let string = map_to_csl(tags);
    xattr::set(path, XATTR_NAME, &string.as_bytes()).map_err(|err| XTagError::File(err))
}

/// Delete all tags for file
pub fn delete_tags(path: &PathBuf) -> Result<()> {
    match xattr::remove(path, XATTR_NAME) {
        Ok(()) => Ok(()),
        Err(err) if err.to_string().starts_with("No data available") => Ok(()),
        Err(err) => Err(XTagError::File(err)),
    }
}

pub fn rename(find: &str, replace: &str, tags: XTags) -> Result<XTags> {
    let mut result: XTags = HashMap::with_capacity(tags.len());
    let re = Regex::new(&searcher::expand_regex(find)).map_err(|err| XTagError::Regex(err))?;
    for (key, value) in tags {
        let new_key = re.replace_all(&key, replace).into_owned();
        result.insert(new_key, value);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::rename;
    use super::XTags;
    use std::collections::HashMap;

    fn test(key: &str, value: &str, from: &str, to: &str, end_key: &str) {
        let mut map: XTags = HashMap::new();
        map.insert(key.to_string(), Some(value.to_string()));
        let map = rename(from, to, map).unwrap();
        assert_eq!(map.len(), 1);
        let element = map.iter().next().unwrap();
        assert_eq!(element.0, end_key);
        assert_eq!(element.1, &Some(value.to_string()));
    }

    #[test]
    fn rename_supports_plain_text() {
        test("from", "value", "from", "to", "to");
    }

    #[test]
    fn rename_supports_regex() {
        test("from", "value", "f(.)om", "to$1", "tor");
    }

    #[test]
    fn rename_supports_named_capture_groups() {
        test("from", "value", "f(?P<a>.)om", "to$a", "tor");
    }
}
