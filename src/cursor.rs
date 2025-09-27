#[derive(Debug)]
pub struct Cursor<'a> {
    buf: &'a [u8],
    i: usize,
}

impl<'a> Cursor<'a> {
    #[inline]
    pub const fn new(buf: &'a [u8]) -> Self {
        Self { buf, i: 0 }
    }

    // Basic queries
    #[inline]
    pub fn eof(&self) -> bool {
        self.i >= self.buf.len()
    }
    #[inline]
    pub fn pos(&self) -> usize {
        self.i
    }
    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.i)
    }
    #[inline]
    pub fn as_slice(&self) -> &'a [u8] {
        &self.buf[self.i..]
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    // Peek/consume
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.buf.get(self.i).copied()
    }
    #[inline]
    pub fn peek_n(&self, n: usize) -> Option<u8> {
        self.buf.get(self.i + n).copied()
    }
    #[inline]
    pub fn next_byte(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        Some(b)
    }
    #[inline]
    pub fn advance(&mut self, n: usize) -> usize {
        let rem = self.remaining();
        let step = if n > rem { rem } else { n };
        self.i += step;
        step
    }
    #[inline]
    pub fn skip_byte(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    // Bookmarking
    #[inline]
    pub fn mark(&self) -> usize {
        self.i
    }
    #[inline]
    pub fn reset(&mut self, m: usize) {
        self.i = m.min(self.buf.len());
    }

    // ASCII whitespace utilities
    #[inline]
    pub const fn is_space_ascii(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'\x0C' | b'\x0B')
    }
    #[inline]
    pub fn skip_space(&mut self) -> usize {
        let start = self.i;
        while let Some(&b) = self.buf.get(self.i) {
            if !Self::is_space_ascii(b) {
                break;
            }
            self.i += 1;
        }
        self.i - start
    }

    // Scanning and matching
    // Skip until delimiter b or EOF; does not consume the delimiter.
    #[inline]
    pub fn skip_until(&mut self, b: u8) -> usize {
        if let Some(off) = self.buf[self.i..].iter().position(|&x| x == b) {
            self.i += off;
            off
        } else {
            let rem = self.remaining();
            self.i = self.buf.len();
            rem
        }
    }

    // Match a byte sequence; consumes on success.
    #[inline]
    pub fn match_bytes(&mut self, pat: &[u8]) -> bool {
        if self.buf[self.i..].starts_with(pat) {
            self.i += pat.len();
            true
        } else {
            false
        }
    }

    // Expect a byte sequence; consumes on success, otherwise rolls back.
    #[inline]
    pub fn expect_bytes(&mut self, pat: &[u8]) -> bool {
        let m = self.mark();
        if self.match_bytes(pat) {
            true
        } else {
            self.reset(m);
            false
        }
    }

    // Take ASCII word [A-Za-z0-9_]+; returns slice range as (start, end)
    #[inline]
    pub fn take_ident_ascii(&mut self) -> Option<(usize, usize)> {
        let start = self.i;
        while let Some(&b) = self.buf.get(self.i) {
            let is_ident = b.is_ascii_alphanumeric() || b == b'_';
            if !is_ident {
                break;
            }
            self.i += 1;
        }
        if self.i > start {
            Some((start, self.i))
        } else {
            None
        }
    }

    #[inline]
    pub const fn is_ident_start_ascii(b: u8) -> bool {
        matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'_')
    }
    #[inline]
    pub const fn is_ident_continue_ascii(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    // Ident starting with letter/_ then [A-Za-z0-9_]*. Returns (start, end).
    #[inline]
    pub fn take_ident_starting_alpha(&mut self) -> Option<(usize, usize)> {
        let start = self.i;
        match self.peek() {
            Some(b) if Self::is_ident_start_ascii(b) => self.i += 1,
            _ => return None,
        }
        while let Some(&b) = self.buf.get(self.i) {
            if !Self::is_ident_continue_ascii(b) {
                break;
            }
            self.i += 1;
        }
        Some((start, self.i))
    }

    // Decimal integer: [0-9]+
    #[inline]
    pub fn take_int_ascii(&mut self) -> Option<(usize, usize)> {
        let start = self.i;
        if !matches!(self.peek(), Some(b'0'..=b'9')) {
            return None;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.i += 1;
        }
        Some((start, self.i))
    }

    // Skip while predicate holds; returns bytes skipped.
    #[inline]
    pub fn skip_while(&mut self, mut pred: impl FnMut(u8) -> bool) -> usize {
        let start = self.i;
        while let Some(&b) = self.buf.get(self.i) {
            if !pred(b) {
                break;
            }
            self.i += 1;
        }
        self.i - start
    }

    // Take while predicate holds; returns (start, end).
    #[inline]
    pub fn take_while(&mut self, mut pred: impl FnMut(u8) -> bool) -> Option<(usize, usize)> {
        let start = self.i;
        while let Some(&b) = self.buf.get(self.i) {
            if !pred(b) {
                break;
            }
            self.i += 1;
        }
        if self.i > start {
            Some((start, self.i))
        } else {
            None
        }
    }

    // Expect a single byte with rollback on failure.
    #[inline]
    pub fn expect_byte(&mut self, b: u8) -> bool {
        let m = self.mark();
        if self.skip_byte(b) {
            true
        } else {
            self.reset(m);
            false
        }
    }

    // Advance until an unescaped delimiter; does not consume the delimiter.
    // Returns bytes advanced and whether delimiter was found.
    #[inline]
    pub fn take_until_unescaped(&mut self, delim: u8, esc: u8) -> (usize, bool) {
        let start = self.i;
        while let Some(b) = self.peek() {
            if b == esc {
                self.i += 1; // skip escape
                if !self.eof() {
                    self.i += 1;
                } // skip escaped char
                continue;
            }
            if b == delim {
                break;
            }
            self.i += 1;
        }
        (self.i - start, self.peek() == Some(delim))
    }

    // Like above, but stops on either a or b; returns which delimiter if found.
    #[inline]
    pub fn take_until_unescaped2(&mut self, a: u8, b: u8, esc: u8) -> (usize, Option<u8>) {
        let start = self.i;
        while let Some(ch) = self.peek() {
            if ch == esc {
                self.i += 1;
                if !self.eof() {
                    self.i += 1;
                }
                continue;
            }
            if ch == a || ch == b {
                break;
            }
            self.i += 1;
        }
        let found = match self.peek() {
            Some(x) if x == a || x == b => Some(x),
            _ => None,
        };
        (self.i - start, found)
    }

    // Non-consuming peek for a slice prefix.
    #[inline]
    pub fn starts_with(&self, pat: &[u8]) -> bool {
        self.buf[self.i..].starts_with(pat)
    }

    // Peek a slice of length n from current pos.
    #[inline]
    pub fn peek_slice(&self, n: usize) -> Option<&'a [u8]> {
        self.buf.get(self.i..self.i + n)
    }
}

// Allow idiomatic iteration over bytes: for b in cursor { ... }
impl<'a> Iterator for Cursor<'a> {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.next_byte()
    }
}
