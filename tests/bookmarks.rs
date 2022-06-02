use xtag;

fn find_in_string(term: &str, string: &str) -> bool {
    dbg!(term);
    let tags = xtag::csl_to_map(string).unwrap();
    let searcher = xtag::compile_search(term).unwrap();
    searcher.is_match(&tags)
}

#[test]
fn grammar_bookmarks_have_implicit_parentheses() {
    assert_eq!(find_in_string("{tests/a_or_b} and c", "a,c"), true);
    assert_eq!(find_in_string("{tests/a_or_b} and c", "c"), false);
    assert_eq!(find_in_string("{tests/a_or_b} and c", "a"), false);
}
