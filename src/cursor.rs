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
    pub fn advance(&mut self, n: usize) -> Option<(usize, usize)> {
        if n > self.remaining() {
            return None;
        }
        let start = self.i;
        self.i += n;
        Some((start, self.i))
    }
    #[inline]
    pub fn skip_byte(&mut self, b: u8) -> Option<(usize, usize)> {
        let start = self.i;
        if self.peek()? == b {
            self.i += 1;
            Some((start, self.i))
        } else {
            None
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
    #[inline]
    pub fn slice_from(&self, m: usize) -> &'a [u8] {
        &self.buf[m.min(self.buf.len())..self.i.min(self.buf.len())]
    }

    // ASCII whitespace
    #[inline]
    pub const fn is_space_ascii(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'\x0C' | b'\x0B')
    }
    #[inline]
    pub fn take_space(&mut self) -> Option<(usize, usize)> {
        self.take_while(Self::is_space_ascii)
    }

    // Scanning and matching
    // Skip until delimiter b or EOF; does not consume the delimiter.
    #[inline]
    pub fn skip_until(&mut self, b: u8) -> (usize, usize) {
        let start = self.i;
        if let Some(off) = self.buf[self.i..].iter().position(|&x| x == b) {
            self.i += off;
        } else {
            self.i = self.buf.len();
        }
        (start, self.i)
    }

    // Match a byte sequence; consumes on success.
    #[inline]
    pub fn match_bytes(&mut self, pat: &[u8]) -> Option<(usize, usize)> {
        let start = self.i;
        if self.buf[self.i..].starts_with(pat) {
            self.i += pat.len();
            Some((start, self.i))
        } else {
            None
        }
    }

    // Expect a byte sequence; consumes on success, otherwise rolls back.
    #[inline]
    pub fn expect_bytes(&mut self, pat: &[u8]) -> Option<(usize, usize)> {
        let m = self.mark();
        match self.match_bytes(pat) {
            Some(span) => Some(span),
            None => {
                self.reset(m);
                None
            }
        }
    }

    // Identifiers and numbers
    #[inline]
    pub const fn is_ident_start_ascii(b: u8) -> bool {
        matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'_')
    }
    #[inline]
    pub const fn is_ident_continue_ascii(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    // [A-Za-z0-9_]+
    #[inline]
    pub fn take_ident_ascii(&mut self) -> Option<(usize, usize)> {
        self.take_while(Self::is_ident_continue_ascii)
    }

    // [A-Za-z_][A-Za-z0-9_]*
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

    // [0-9]+
    #[inline]
    pub fn take_int_ascii(&mut self) -> Option<(usize, usize)> {
        self.take_while(|b| b.is_ascii_digit())
    }

    // Predicate-based
    #[inline]
    pub fn skip_while(&mut self, mut pred: impl FnMut(u8) -> bool) -> (usize, usize) {
        let start = self.i;
        while let Some(&b) = self.buf.get(self.i) {
            if !pred(b) {
                break;
            }
            self.i += 1;
        }
        (start, self.i)
    }
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

    // Single-byte expectation with rollback
    #[inline]
    pub fn expect_byte(&mut self, b: u8) -> Option<(usize, usize)> {
        let m = self.mark();
        if let Some(span) = self.skip_byte(b) {
            Some(span)
        } else {
            self.reset(m);
            None
        }
    }

    // Prefix/slice peeking
    #[inline]
    pub fn starts_with(&self, pat: &[u8]) -> bool {
        self.buf[self.i..].starts_with(pat)
    }
    #[inline]
    pub fn peek_slice(&self, n: usize) -> Option<&'a [u8]> {
        self.buf.get(self.i..self.i + n)
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.next_byte()
    }
}
