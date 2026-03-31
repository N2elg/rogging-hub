use RoggingHub::io_handler::framer::JsonFramer;

fn extract(input: &[u8]) -> (Vec<(usize, usize)>, JsonFramer) {
    let mut f = JsonFramer::new();
    let positions = f.extract_positions(input);
    (positions, f)
}

fn extract_strings(input: &[u8]) -> Vec<String> {
    let (positions, _) = extract(input);
    positions
        .iter()
        .map(|&(s, e)| String::from_utf8_lossy(&input[s..e]).to_string())
        .collect()
}

#[test]
fn single_object() {
    let input = br#"{"key":"value"}"#;
    let (positions, f) = extract(input);
    assert_eq!(positions, vec![(0, 15)]);
    assert_eq!(f.depth, 0);
}

#[test]
fn multiple_objects_sticky_packet() {
    let input = br#"{"a":1}{"b":2}{"c":3}"#;
    let strs = extract_strings(input);
    assert_eq!(strs, vec![r#"{"a":1}"#, r#"{"b":2}"#, r#"{"c":3}"#]);
}

#[test]
fn objects_with_whitespace_between() {
    let input = b"  {\"a\":1}  \n  {\"b\":2}  ";
    let strs = extract_strings(input);
    assert_eq!(strs, vec![r#"{"a":1}"#, r#"{"b":2}"#]);
}

#[test]
fn nested_braces() {
    let input = br#"{"outer":{"inner":{"deep":1}}}"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert_eq!(f.depth, 0);
    assert_eq!(&input[positions[0].0..positions[0].1], input.as_slice());
}

#[test]
fn braces_inside_strings_ignored() {
    let input = br#"{"msg":"hello { world } }"}"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert_eq!(f.depth, 0);
}

#[test]
fn escaped_quotes_in_strings() {
    let input = br#"{"msg":"say \"hello\" {}"}"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert_eq!(f.depth, 0);
}

#[test]
fn escaped_backslash_before_quote() {
    let input = br#"{"val":"a\\"}"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert_eq!(f.depth, 0);
}

#[test]
fn half_packet_incomplete_object() {
    let input = br#"{"key":"val"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 0);
    assert!(f.depth > 0 || f.in_string);
}

#[test]
fn half_packet_then_complete() {
    let mut f = JsonFramer::new();

    let chunk1 = br#"{"key":"va"#;
    let p1 = f.extract_positions(chunk1);
    assert_eq!(p1.len(), 0);

    let chunk2 = br#"lue"}"#;
    let p2 = f.extract_positions(chunk2);
    assert_eq!(p2.len(), 1);
    assert_eq!(f.depth, 0);
}

#[test]
fn shift_adjusts_start() {
    let mut f = JsonFramer::new();
    f.start = 10;
    f.shift(5);
    assert_eq!(f.start, 5);

    f.start = 3;
    f.shift(10);
    assert_eq!(f.start, 0);
}

#[test]
fn empty_input() {
    let (positions, f) = extract(b"");
    assert!(positions.is_empty());
    assert_eq!(f.depth, 0);
}

#[test]
fn only_whitespace() {
    let (positions, f) = extract(b"   \n\r\t  ");
    assert!(positions.is_empty());
    assert_eq!(f.depth, 0);
}

#[test]
fn unexpected_closing_brace_resets() {
    let input = br#"}"#;
    let (positions, f) = extract(input);
    assert!(positions.is_empty());
    assert_eq!(f.depth, 0);
}

#[test]
fn mixed_complete_and_trailing_incomplete() {
    let input = br#"{"a":1}{"b":"incomp"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert!(f.depth > 0 || f.in_string);
}

#[test]
fn multiline_json() {
    let input = b"{\n  \"key\": \"value\",\n  \"num\": 42\n}";
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert_eq!(f.depth, 0);
}

#[test]
fn logstash_style_json() {
    let input = br#"{"@timestamp":"2026-04-01T12:00:00.000Z","@version":"1","message":"test","host":{"name":"server1"},"log":{"level":"INFO"}}"#;
    let (positions, f) = extract(input);
    assert_eq!(positions.len(), 1);
    assert_eq!(f.depth, 0);
}
