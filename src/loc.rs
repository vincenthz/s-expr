use std::fmt;

/// A file position for human composed of the line (starting at 1), and column (starting a 0)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self { line: 1, col: 0 }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl Position {
    pub fn advance_line(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    pub fn advance_col(&mut self) {
        self.col += 1;
    }

    pub fn advance(&mut self, c: char) {
        if c == '\n' {
            self.advance_line()
        } else {
            self.advance_col()
        }
    }
}

/// Span defined by 2 positions, defining a range between start and end
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn extend(&self, other: &Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }

    pub fn on_line(line: usize, start_col: usize, end_col: usize) -> Self {
        Self {
            start: Position {
                line,
                col: start_col,
            },
            end: Position { line, col: end_col },
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// A type with the span (start and end positions) associated
#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub span: Span,
    pub inner: T,
}
