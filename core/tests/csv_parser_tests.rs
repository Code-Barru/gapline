use std::io::BufReader;

use gapline_core::parser::csv_parser::parse_csv;

#[test]
fn basic_parsing() {
    let data = b"col_a,col_b\nfoo,bar\nbaz,qux\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();

    let (line, row) = iter.next_row().unwrap();
    assert_eq!(line, 2); // line 1 = headers, line 2 = first data row
    assert_eq!(row.get("col_a"), Some("foo"));
    assert_eq!(row.get("col_b"), Some("bar"));

    let (line, row) = iter.next_row().unwrap();
    assert_eq!(line, 3);
    assert_eq!(row.get("col_a"), Some("baz"));

    assert!(iter.next_row().is_none());
}

#[test]
fn bom_stripping() {
    let data = b"\xEF\xBB\xBFagency_id,agency_name\nSTM,STM\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();

    assert_eq!(iter.headers()[0], "agency_id");

    let (_, row) = iter.next_row().unwrap();
    assert_eq!(row.get("agency_id"), Some("STM"));
}

#[test]
fn flexible_row_length() {
    let data = b"a,b,c\n1,2\n1,2,3,4\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();

    let (_, row) = iter.next_row().unwrap();
    assert_eq!(row.get("c"), None);

    let (_, row) = iter.next_row().unwrap();
    assert_eq!(row.get("c"), Some("3"));

    assert!(iter.next_row().is_none());
}

#[test]
fn quoted_values() {
    let data = b"name,desc\n\"hello, world\",\"line1\nline2\"\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();

    let (_, row) = iter.next_row().unwrap();
    assert_eq!(row.get("name"), Some("hello, world"));
    assert!(row.get("desc").unwrap().contains('\n'));

    assert!(iter.next_row().is_none());
}

#[test]
fn empty_file_headers_only() {
    let data = b"a,b,c\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();
    assert!(iter.next_row().is_none());
}

#[test]
fn unknown_column_ignored() {
    let data = b"agency_id,agency_color\nSTM,#FF0000\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();

    let (_, row) = iter.next_row().unwrap();
    assert_eq!(row.get("agency_color"), Some("#FF0000"));
}
