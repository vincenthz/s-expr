use super::data::{Atom, GroupKind};
use super::loc::{Position, Span, Spanned};
use super::tokenizer::{Token, TokenError, Tokenizer, TokenizerConfig};

/// Element of S-Expr
#[derive(Debug, Clone)]
pub enum Element<'a> {
    Group(GroupKind, Vec<SpannedElement<'a>>),
    Atom(Atom<'a>),
    Comment(&'a str),
}

impl<'a> Element<'a> {
    /// Return the atom if the element is an atom, otherwise None
    pub fn atom(&self) -> Option<&Atom<'a>> {
        match self {
            Element::Group(_, _) => None,
            Element::Comment(_) => None,
            Element::Atom(a) => Some(a),
        }
    }

    /// Return the group elements if the element is a group of the right type, otherwise None
    pub fn group(&self, grp: GroupKind) -> Option<&[SpannedElement<'a>]> {
        match self {
            Element::Group(got_grp, elements) if *got_grp == grp => Some(elements),
            Element::Group(_, _) => None,
            Element::Comment(_) => None,
            Element::Atom(_) => None,
        }
    }

    /// Return the group elements if the element is a paren group, otherwise None
    pub fn paren(&self) -> Option<&[SpannedElement<'a>]> {
        self.group(GroupKind::Paren)
    }

    /// Return the group elements if the element is a bracket group, otherwise None
    pub fn bracket(&self) -> Option<&[SpannedElement<'a>]> {
        self.group(GroupKind::Bracket)
    }

    /// Return the group elements if the element is a brace group, otherwise None
    pub fn brace(&self) -> Option<&[SpannedElement<'a>]> {
        self.group(GroupKind::Brace)
    }
}

/// Spanned Element
pub type SpannedElement<'a> = Spanned<Element<'a>>;

/// S-Expr Parser
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

/// Parser Error, which are either token error or some error related to group balancing
/// like unterminated group, or mismatch of group
#[derive(Debug, Clone)]
pub enum ParserError {
    UnbalancedEmpty(Position, GroupKind),
    UnbalancedMismatch {
        span: Span,
        expected: GroupKind,
        got: GroupKind,
    },
    UnfinishedGroup(GroupKind),
    TokenizerError(TokenError),
}

impl From<TokenError> for ParserError {
    fn from(t: TokenError) -> ParserError {
        ParserError::TokenizerError(t)
    }
}

impl<'a> Parser<'a> {
    pub fn new_with_config(data: &'a str, cfg: TokenizerConfig) -> Self {
        Parser {
            tokenizer: Tokenizer::new_with_config(data, cfg),
        }
    }

    pub fn new(data: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(data),
        }
    }

    pub fn next(&mut self) -> Result<Option<SpannedElement<'a>>, ParserError> {
        let mut out: Vec<(GroupKind, Span, Vec<SpannedElement<'a>>)> = vec![];
        loop {
            match self.tokenizer.next()? {
                None => match out.last() {
                    None => return Ok(None),
                    Some((grp, _, _)) => return Err(ParserError::UnfinishedGroup(*grp)),
                },
                Some(tok) => match tok.inner {
                    Token::Comment(comment) => {
                        let el = Spanned {
                            span: tok.span,
                            inner: Element::Comment(comment),
                        };
                        match out.last_mut() {
                            None => return Ok(Some(el)),
                            Some((_, _, elements)) => {
                                elements.push(el);
                            }
                        }
                    }
                    Token::Atom(atom) => {
                        let el = Spanned {
                            span: tok.span,
                            inner: Element::Atom(atom),
                        };
                        match out.last_mut() {
                            None => return Ok(Some(el)),
                            Some((_, _, elements)) => {
                                elements.push(el);
                            }
                        }
                    }
                    Token::Left(grp) => {
                        // create a new group
                        out.push((grp, tok.span, Vec::new()));
                    }
                    Token::Right(grp) => match out.pop() {
                        None => {
                            return Err(ParserError::UnbalancedEmpty(tok.span.start, grp));
                        }
                        Some((inner_grp, inner_start, inner_elements)) => {
                            if inner_grp != grp {
                                return Err(ParserError::UnbalancedMismatch {
                                    span: inner_start.extend(&tok.span),
                                    expected: inner_grp,
                                    got: grp,
                                });
                            }
                            let inner = Spanned {
                                span: inner_start.extend(&tok.span),
                                inner: Element::Group(grp, inner_elements),
                            };
                            match out.last_mut() {
                                None => return Ok(Some(inner)),
                                Some((_, _, elements)) => {
                                    elements.push(inner);
                                }
                            }
                        }
                    },
                },
            }
        }
    }
}
