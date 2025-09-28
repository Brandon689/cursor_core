use cursor_core::cursor::Cursor;
use core::ops::Range;

/// Assert that an Option<Range> is Some and has the given length.
fn assert_span_len(span: Option<Range<usize>>, len: usize) -> Range<usize> {
    let s = span.expect("expected Some(span)");
    assert_eq!(s.len(), len);
    s
}

/// Assert that an Option<Range> is None (no span returned).
fn assert_no_span(span: Option<Range<usize>>) {
    assert!(span.is_none());
}

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

    assert_span_len(c.take_space(), 2);

    let ident = c.take_ident_ascii().expect("ident");
    assert_eq!(&input[ident.clone()], b"foo_bar123");
    assert_eq!(c.pos(), ident.end);
}

#[test]
fn match_and_expect_bytes() {
    let mut d = Cursor::new(b"### title");
    assert!(d.expect_bytes(b"###").is_some());

    assert_span_len(d.take_space(), 1);
    assert_eq!(std::str::from_utf8(d.as_slice()).unwrap(), "title");

    let mut e = Cursor::new(b"abc");
    assert_no_span(e.expect_bytes(b"zzz"));
    assert_eq!(e.pos(), 0); // rolled back
}

#[test]
fn skip_until_and_skip_byte() {
    let mut e = Cursor::new(b"name=value; rest");

    let skipped = e.skip_until(b'=');
    assert_eq!(skipped.len(), "name".len());

    assert!(e.skip_byte(b'=').is_some());
    assert_eq!(std::str::from_utf8(e.as_slice()).unwrap(), "value; rest");

    assert_no_span(e.take_space()); // no space to skip
    assert_eq!(std::str::from_utf8(e.as_slice()).unwrap(), "value; rest");
}

#[test]
fn advance_and_remaining() {
    let mut c = Cursor::new(b"abc");
    assert_eq!(c.remaining(), 3);

    // Too large: cannot advance, returns None and position unchanged
    assert_no_span(c.advance(10));
    assert_eq!(c.pos(), 0);
    assert!(!c.eof());

    // Consume exactly remaining
    let adv = assert_span_len(c.advance(3), 3);
    assert_eq!(adv, 0..3);
    assert!(c.eof());

    // Already EOF, can't advance any further
    assert_no_span(c.advance(1));
}

#[test]
fn mark_and_reset() {
    let mut c = Cursor::new(b"12345");
    assert_span_len(c.advance(2), 2);
    let m = c.mark();
    assert_eq!(c.pos(), 2);

    // Too far, returns None but cursor remains unchanged at pos=2
    assert_no_span(c.advance(10));
    assert!(!c.eof());
    assert_eq!(c.pos(), 2);

    c.reset(m);
    assert_eq!(c.pos(), 2);
    assert_eq!(c.peek(), Some(b'3'));
}
