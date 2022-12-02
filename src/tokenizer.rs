use super::data::*;
use super::loc::{Position, Span, Spanned};
use super::utf8::{next_char, MovementInBytes, NextCharError};

#[cfg(feature = "unicode")]
use unicode_xid::UnicodeXID;

/// Config for the tokenizer, for flags
#[derive(Debug, Clone)]
pub struct TokenizerConfig {
    /// Tokenize the comment
    filter_comment: bool,
    /// Add support for the bytes token, which is of the format `#<hexadecimal>#`. Default is set to true
    support_bytes: bool,
    /// Add support for the { } group, Default is set to true
    support_brace: bool,
    /// Add support for the [ ] group, Default is set to true
    support_bracket: bool,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        TokenizerConfig {
            filter_comment: false,
            support_bytes: true,
            support_bracket: true,
            support_brace: true,
        }
    }
}

impl TokenizerConfig {
    /// Support comment in the output of the tokenizer or filter them away
    pub fn comment(mut self, enabled: bool) -> Self {
        self.filter_comment = !enabled;
        self
    }

    /// Support braces group in the output of the tokenizer or filter them away
    pub fn braces(mut self, enabled: bool) -> Self {
        self.support_brace = enabled;
        self
    }

    /// Support bracket group in the output of the tokenizer or filter them away
    pub fn bracket(mut self, enabled: bool) -> Self {
        self.support_bracket = enabled;
        self
    }

    /// Support the bytes atom in the output of the tokenizer
    pub fn support_bytes(mut self, supported: bool) -> Self {
        self.support_bytes = supported;
        self
    }
}

/// Tokenizer state on the data
pub struct Tokenizer<'a> {
    data: &'a [u8],
    index: TokDataPos,
    position: Position,
    cfg: TokenizerConfig,
}

#[derive(Clone, Copy)]
pub struct TokDataPos(usize);

/// Tokens
#[derive(Clone, Debug)]
pub enum Token<'a> {
    /// Left group
    Left(GroupKind),
    /// Right group
    Right(GroupKind),
    /// Comment starting with ';'
    Comment(&'a str),
    /// Atom
    Atom(Atom<'a>),
}

impl<'a> Token<'a> {
    pub fn is_comment(&self) -> bool {
        match self {
            Token::Comment(_) => true,
            _ => false,
        }
    }
}

/// A Token with the span (start and end positions) associated
pub type SpannedToken<'a> = Spanned<Token<'a>>;

#[derive(Clone, Debug)]
pub enum TokenError {
    DataError(NextCharError, usize),
    UnterminatedString(Position),
    UnterminatedBytes(Position),
    UnprocessedChar(char),
    UnterminatedBytesChar(Position, char),
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer from the data stream
    pub fn new(data: &'a str) -> Self {
        Tokenizer {
            data: data.as_bytes(),
            index: TokDataPos(0),
            position: Position::default(),
            cfg: TokenizerConfig::default(),
        }
    }

    /// Create a new tokenizer from the data stream with an associated config
    pub fn new_with_config(data: &'a str, cfg: TokenizerConfig) -> Self {
        Tokenizer {
            data: data.as_bytes(),
            index: TokDataPos(0),
            position: Position::default(),
            cfg,
        }
    }

    /// Return the next token, or none if reach the end of stream
    pub fn next(&mut self) -> Result<Option<SpannedToken<'a>>, TokenError> {
        // note that the tokenizer only take `str` type, so that the content is always invalid,
        // short of an internal error, so all the .expect should not never trigger except on a
        // internal bug.
        loop {
            self.skip_whitespace().expect("Valid string");
            match self.peek_char().expect("Valid string") {
                None => return Ok(None),
                Some((leading_char, advance)) => {
                    let token_start = self.position;
                    let position_start = self.index;
                    self.position.advance(leading_char);
                    self.move_index(advance);
                    let tok = self.next_cont(token_start, position_start, leading_char)?;
                    // if it's a comment, and we filter comment, we don't return
                    if !tok.inner.is_comment() {
                        return Ok(Some(tok));
                    } else {
                        if !self.cfg.filter_comment {
                            return Ok(Some(tok));
                        }
                    }
                }
            }
        }
    }

    fn move_index(&mut self, bytes: MovementInBytes) {
        self.index.0 += bytes.0
    }

    fn slice_from(&self, start: TokDataPos) -> &'a str {
        let slice = &self.data[start.0..self.index.0];
        core::str::from_utf8(slice).expect("valid utf8")
    }

    fn peek_char(&self) -> Result<Option<(char, MovementInBytes)>, TokenError> {
        match next_char(self.data, self.index.0) {
            Err(e) => Err(TokenError::DataError(e, self.index.0)),
            Ok(ok) => Ok(ok),
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), TokenError> {
        loop {
            match self.peek_char()? {
                None => return Ok(()),
                Some((ch, advance)) => {
                    if !"\n\t ".contains(ch) {
                        return Ok(());
                    }
                    self.position.advance(ch);
                    self.move_index(advance);
                }
            }
        }
    }

    /// advance the data stream until the function F return true
    fn skip_until<F>(&mut self, f: F) -> Result<(), TokenError>
    where
        F: Fn(char) -> bool,
    {
        loop {
            match self.peek_char()? {
                None => return Ok(()),
                Some((ch, advance)) => {
                    if f(ch) {
                        return Ok(());
                    }
                    self.position.advance(ch);
                    self.move_index(advance);
                }
            }
        }
    }

    /// advance the data stream while the function F return true
    fn skip_while<F>(&mut self, f: F) -> Result<(), TokenError>
    where
        F: Fn(char) -> bool,
    {
        loop {
            match self.peek_char()? {
                None => return Ok(()),
                Some((ch, advance)) => {
                    if !f(ch) {
                        return Ok(());
                    }
                    self.position.advance(ch);
                    self.move_index(advance);
                }
            }
        }
    }

    fn bytes(&mut self) -> Result<ABytes<'a>, TokenError> {
        let position_start = self.index;
        self.skip_while(|c| c.is_ascii_hexdigit())?;
        match self.peek_char()? {
            None => Err(TokenError::UnterminatedBytes(self.position)),
            Some((ch, advance)) => {
                if ch == '#' {
                    let dat = self.slice_from(position_start);

                    // consume the "
                    self.position.advance(ch);
                    self.move_index(advance);

                    return Ok(ABytes(dat));
                } else {
                    return Err(TokenError::UnterminatedBytesChar(self.position, ch));
                }
            }
        }
    }

    fn number(
        &mut self,
        leading_char: char,
        position_start: TokDataPos,
    ) -> Result<ANum<'a>, TokenError> {
        match self.peek_char()? {
            None => {
                // if we reach the end of stream, just take the current buffer and raise the event
                let dat = self.slice_from(position_start);
                Ok(ANum {
                    base: ANumBase::Decimal,
                    dat: dat,
                })
            }
            Some((ch, advance)) => {
                let zero_start = leading_char == '0';

                if zero_start {
                    if ch == 'b' {
                        // binary string, eat the 'b', and save the initial position
                        self.position.advance(ch);
                        self.move_index(advance);

                        let position_start = self.index;

                        self.skip_while(|c| c == '0' || c == '1' || c == '_')?;
                        Ok(ANum {
                            base: ANumBase::Binary,
                            dat: self.slice_from(position_start),
                        })
                    } else if ch == 'x' {
                        // hexadecimal string, eat the 'x', and save the initial position
                        self.position.advance(ch);
                        self.move_index(advance);

                        let position_start = self.index;

                        self.skip_while(|c| c.is_ascii_hexdigit() || c == '_')?;
                        Ok(ANum {
                            base: ANumBase::Hexadecimal,
                            dat: self.slice_from(position_start),
                        })
                    } else if ch.is_ascii_digit() {
                        self.position.advance(ch);
                        self.move_index(advance);

                        self.skip_while(|c| c.is_numeric() || c == '_')?;
                        Ok(ANum {
                            base: ANumBase::Decimal,
                            dat: self.slice_from(position_start),
                        })
                    } else {
                        let dat = self.slice_from(position_start);
                        Ok(ANum {
                            base: ANumBase::Decimal,
                            dat: dat,
                        })
                    }
                } else {
                    if ch.is_ascii_digit() {
                        self.position.advance(ch);
                        self.move_index(advance);

                        self.skip_while(|c| c.is_numeric() || c == '_')?;
                        Ok(ANum {
                            base: ANumBase::Decimal,
                            dat: self.slice_from(position_start),
                        })
                    } else {
                        let dat = self.slice_from(position_start);
                        Ok(ANum {
                            base: ANumBase::Decimal,
                            dat: dat,
                        })
                    }
                }
            }
        }
    }

    // consume the data
    fn string(&mut self) -> Result<AStr<'a>, TokenError> {
        let mut has_escape = false; // check if there's any escape in the data
        let position_start = self.index;

        let mut escape = false;
        loop {
            match self.peek_char()? {
                None => return Err(TokenError::UnterminatedString(self.position)),
                Some((ch, advance)) => {
                    if escape {
                        escape = false;
                    } else {
                        if ch == '\\' {
                            has_escape = true;
                            escape = true;
                        } else if ch == '"' {
                            let dat = self.slice_from(position_start);

                            // consume the "
                            self.position.advance(ch);
                            self.move_index(advance);

                            return Ok(AStr {
                                has_escape,
                                raw_data: dat,
                            });
                        }
                    }
                    self.position.advance(ch);
                    self.move_index(advance);
                }
            }
        }
    }

    // this method has to parse a token (or return an error)
    fn next_cont(
        &mut self,
        token_start: Position,
        position_start: TokDataPos,
        leading_char: char,
    ) -> Result<SpannedToken<'a>, TokenError> {
        let stok = |cur, token| {
            let span = Span {
                start: token_start,
                end: cur,
            };
            Ok(Spanned { span, inner: token })
        };

        // lex in this order:
        // * group characters: '(' ')' '[' ']' '{' '}'
        // * line comment: ';'
        // * string : '"'
        // * (optionally) bytes : '#'
        // * number : '0'..'9'
        // * identifier : anything else

        if leading_char == '(' {
            stok(self.position, Token::Left(GroupKind::Paren))
        } else if leading_char == ')' {
            stok(self.position, Token::Right(GroupKind::Paren))
        } else if self.cfg.support_bracket && leading_char == '[' {
            stok(self.position, Token::Left(GroupKind::Bracket))
        } else if self.cfg.support_bracket && leading_char == ']' {
            stok(self.position, Token::Right(GroupKind::Bracket))
        } else if self.cfg.support_brace && leading_char == '{' {
            stok(self.position, Token::Left(GroupKind::Brace))
        } else if self.cfg.support_brace && leading_char == '}' {
            stok(self.position, Token::Right(GroupKind::Brace))
        } else if leading_char == ';' {
            // comment
            self.skip_until(|c| c == '\n')?;
            let comment = self.slice_from(position_start);
            stok(self.position, Token::Comment(comment))
        } else if leading_char == '"' {
            // string
            let astr = self.string()?;
            stok(self.position, Token::Atom(Atom::String(astr)))
        } else if self.cfg.support_bytes && leading_char == '#' {
            // byte stream
            let bstr = self.bytes()?;
            stok(self.position, Token::Atom(Atom::Bytes(bstr)))
        } else if leading_char.is_ascii_digit() {
            // number
            let anum = self.number(leading_char, position_start)?;
            let is_decimal = anum.base == ANumBase::Decimal;
            // if this is a decimal number, then we check if it's followed by a '.', in this case it's a decimal type
            if is_decimal {
                match self.peek_char() {
                    Ok(Some((ch @ '.', dot_advance))) => {
                        self.position.advance(ch);
                        self.move_index(dot_advance);

                        // might parse no decimal part, but we accept it `1.` will be equivalent to `1.0`
                        let fractional_start = self.index;
                        self.skip_while(|c| c.is_ascii_digit())?;
                        let raw_fractional = self.slice_from(fractional_start);

                        let adec = ADecimal {
                            raw_integral: anum.dat,
                            raw_fractional,
                        };
                        stok(self.position, Token::Atom(Atom::Decimal(adec)))
                    }
                    _ => stok(self.position, Token::Atom(Atom::Integral(anum))),
                }
            } else {
                stok(self.position, Token::Atom(Atom::Integral(anum)))
            }
        } else if is_id_start(leading_char) {
            self.skip_while(|c| is_id_continue(c))?;
            let ident = self.slice_from(position_start);
            stok(self.position, Token::Atom(Atom::Ident(ident)))
        } else {
            Err(TokenError::UnprocessedChar(leading_char))
        }
    }
}

fn is_id_start(ch: char) -> bool {
    #[cfg(feature = "unicode")]
    {
        ch.is_xid_start()
            || ch == '_'
            || is_ascii_operator(ch)
            || crate::utf8::extended_math_operator(ch)
    }
    #[cfg(not(feature = "unicode"))]
    {
        ch.is_ascii_alphabetic() || ch == '_' || is_ascii_operator(ch)
    }
}

fn is_id_continue(ch: char) -> bool {
    #[cfg(feature = "unicode")]
    {
        ch.is_xid_continue()
            || ch == '_'
            || ch.is_ascii_digit()
            || is_ascii_operator(ch)
            || crate::utf8::extended_math_operator(ch)
    }
    #[cfg(not(feature = "unicode"))]
    {
        ch.is_ascii_alphabetic() || ch == '_' || ch.is_ascii_digit() || is_ascii_operator(ch)
    }
}

fn is_ascii_operator(ch: char) -> bool {
    // any ascii operator except: [] {} () " ; \\
    "?!#@$+-*/=<>,.:|%^&~'`".contains(ch)
}
