use std::io::BufReader;

use headway_core::parser::csv_parser::parse_csv;

#[test]
fn basic_parsing() {
    let data = b"col_a,col_b\nfoo,bar\nbaz,qux\n";
    let records: Vec<_> = parse_csv(BufReader::new(&data[..])).unwrap().collect();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].0, 2); // line 1 = headers, line 2 = first data row
    assert_eq!(records[0].1["col_a"], "foo");
    assert_eq!(records[0].1["col_b"], "bar");
    assert_eq!(records[1].0, 3);
    assert_eq!(records[1].1["col_a"], "baz");
}

#[test]
fn bom_stripping() {
    let data = b"\xEF\xBB\xBFagency_id,agency_name\nSTM,STM\n";
    let mut iter = parse_csv(BufReader::new(&data[..])).unwrap();

    assert_eq!(iter.headers()[0], "agency_id");

    let (_, row) = iter.next().unwrap();
    assert_eq!(row["agency_id"], "STM");
}

#[test]
fn flexible_row_length() {
    let data = b"a,b,c\n1,2\n1,2,3,4\n";
    let records: Vec<_> = parse_csv(BufReader::new(&data[..])).unwrap().collect();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].1.get("c"), None);
    assert_eq!(records[1].1["c"], "3");
}

#[test]
fn quoted_values() {
    let data = b"name,desc\n\"hello, world\",\"line1\nline2\"\n";
    let records: Vec<_> = parse_csv(BufReader::new(&data[..])).unwrap().collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].1["name"], "hello, world");
    assert!(records[0].1["desc"].contains('\n'));
}

#[test]
fn empty_file_headers_only() {
    let data = b"a,b,c\n";
    let records: Vec<_> = parse_csv(BufReader::new(&data[..])).unwrap().collect();
    assert!(records.is_empty());
}

#[test]
fn unknown_column_ignored() {
    let data = b"agency_id,agency_color\nSTM,#FF0000\n";
    let records: Vec<_> = parse_csv(BufReader::new(&data[..])).unwrap().collect();

    assert_eq!(records[0].1["agency_color"], "#FF0000");
}
