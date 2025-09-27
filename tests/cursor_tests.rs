use cursor_core::cursor::Cursor;

#[test]
fn eof_peek_next() {
    let mut c = Cursor::new(b"ab");
    assert_eq!(c.peek(), Some(b'a'));
    assert_eq!(c.peek_n(1), Some(b'b'));
    assert!(!c.eof());
    assert_eq!(c.next(), Some(b'a'));
    assert_eq!(c.next(), Some(b'b'));
    assert_eq!(c.next(), None);
    assert!(c.eof());
}

#[test]
fn skip_space_and_ident() {
    let input = b"  foo_bar123 ";
    let mut c = Cursor::new(input);
    assert_eq!(c.skip_space(), 2);
    let (s, e) = c.take_ident_ascii().expect("ident");
    assert_eq!(&input[s..e], b"foo_bar123");
    assert_eq!(c.pos(), e);
}

#[test]
fn match_and_expect_bytes() {
    let mut d = Cursor::new(b"### title");
    assert!(d.expect_bytes(b"###"));
    assert_eq!(d.skip_space(), 1);
    assert_eq!(std::str::from_utf8(d.as_slice()).unwrap(), "title");
    let mut e = Cursor::new(b"abc");
    assert!(!e.expect_bytes(b"zzz"));
    assert_eq!(e.pos(), 0); // rolled back
}

#[test]
fn skip_until_and_skip_byte() {
    let mut e = Cursor::new(b"name=value; rest");
    let skipped = e.skip_until(b'=');
    assert_eq!(skipped, "name".len());
    assert!(e.skip_byte(b'='));
    assert_eq!(std::str::from_utf8(e.as_slice()).unwrap(), "value; rest"); // no leading space
    assert_eq!(e.skip_space(), 0); // there is no space to skip here
    assert_eq!(std::str::from_utf8(e.as_slice()).unwrap(), "value; rest");
}

#[test]
fn advance_and_remaining() {
    let mut c = Cursor::new(b"abc");
    assert_eq!(c.remaining(), 3);
    assert_eq!(c.advance(10), 3);
    assert!(c.eof());
    assert_eq!(c.advance(1), 0);
}

#[test]
fn mark_and_reset() {
    let mut c = Cursor::new(b"12345");
    c.advance(2);
    let m = c.mark();
    c.advance(10);
    assert!(c.eof());
    c.reset(m);
    assert_eq!(c.pos(), 2);
    assert_eq!(c.peek(), Some(b'3'));
}
