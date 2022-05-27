use std::fs;
use std::path::PathBuf;

use crate::compile_search;
use crate::error::XTagError;
use crate::Result;
use crate::Searcher;

/// Get bookmark from filesystem
///
/// Bookmark is a symbolic link with the filter term as link
pub fn get_bookmark(path: &PathBuf) -> Result<Searcher> {
    let term = fs::read_link(path)?
        .into_os_string()
        .into_string()
        .map_err(|string| XTagError::Bookmark(string))?;
    compile_search(&term)
}
