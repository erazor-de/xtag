mod error;
mod parse_search;
mod parse_tags;
mod parser;

pub use crate::error::TaggerError;
pub use crate::parse_search::search;
pub use crate::parse_tags::csl_to_map;
use crate::parser::Rule;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str;

static XATTR_NAME: &'static str = "user.xtag";

/// Convert map to comma separated list of tag=value pairs
pub fn map_to_csl(set: &HashMap<String, Option<String>>) -> String {
    set.iter()
        .map(|(tag, value)| match value {
            Some(value) => tag.to_string() + "=" + value,
            None => tag.to_string(),
        })
        .join(",")
}

/// Get tags for file as map
pub fn get_tags(path: &PathBuf) -> Result<HashMap<String, Option<String>>, TaggerError> {
    let xattrs = xattr::get(path, XATTR_NAME).map_err(|err| TaggerError::File(err))?;
    match xattrs {
        Some(value) => {
            let string = str::from_utf8(&value).map_err(|err| TaggerError::Charset(err))?;
            csl_to_map(string)
        }
        None => csl_to_map(""),
    }
}

/// Set tags for file from map
///
/// The used utf-8 string format is architecture independent.
pub fn set_tags(path: &PathBuf, tags: &HashMap<String, Option<String>>) -> Result<(), TaggerError> {
    let string = map_to_csl(tags);
    xattr::set(path, XATTR_NAME, &string.as_bytes()).map_err(|err| TaggerError::File(err))
}

/// Delete all tags for file
pub fn delete_tags(path: &PathBuf) -> Result<(), TaggerError> {
    match xattr::remove(path, XATTR_NAME) {
        Ok(()) => Ok(()),
        Err(err) if err.to_string().starts_with("No data available") => Ok(()),
        Err(err) => Err(TaggerError::File(err)),
    }
}
