//! S-Expressions Parser and Printer
//!
//! # Parser example
//!
//! ```
//! use s_expr::{Parser, Element, Span};
//!
//! let mut parser = Parser::new("(let x 1)");
//! let r = parser.next().expect("parse data").expect("not end of stream");
//!
//! let elements = r.inner.paren().expect("paren group");
//! assert_eq!(elements[0].inner.atom().and_then(|atom| atom.ident()), Some("let"));
//! assert_eq!(elements[0].span, Span::on_line(1, 1, 4));
//! ```

mod data;
mod loc;
mod parser;
mod printer;
mod tokenizer;
mod utf8;

pub use data::{ABytes, ADecimal, ANum, Atom, GroupKind};
pub use loc::{Position, Span};
pub use parser::{Element, Parser, ParserError, SpannedElement};
pub use printer::Printer;
pub use tokenizer::{SpannedToken, Token, TokenError, Tokenizer, TokenizerConfig};

#[cfg(test)]
mod tests {
    use super::*;

    const PROG1: &str = r#"
(define x 1) ; this is a post comment
; this is a comment
(define y 0x01_ab)
(if (zero? x)
    (strip " " "abc")
    [1 2 "def\"x"]
)
"#;

    const PROG2: &str = r#"
    (define hello world 123)
    
    ; comment space
    #1234# ( (let x 1) (let y = x + x) 123 x ) "string"
    
    ( "this is a quote char: \" " )

    (== (/ (+ 1 2) 3) 1)
    (pöjk unicode) ; unicode support
"#;

    fn collect_tokens<'a>(
        mut tokenizer: Tokenizer<'a>,
    ) -> Result<Vec<SpannedToken<'a>>, TokenError> {
        let mut toks = Vec::new();
        loop {
            match tokenizer.next() {
                Ok(Some(tok)) => toks.push(tok),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }
        return Ok(toks);
    }

    #[test]
    fn prog1_tokenize() {
        let toks1 = collect_tokens(Tokenizer::new(PROG1));
        assert!(toks1.is_ok())
    }

    #[test]
    fn prog2_tokenize() {
        let toks2 = collect_tokens(Tokenizer::new(PROG2));
        assert!(toks2.is_ok())
    }

    #[test]
    fn prog1_parser() {
        let mut parser = Parser::new_with_config(PROG1, TokenizerConfig::default().comment(false));
        {
            let first_element = parser
                .next()
                .expect("parser error")
                .expect("not end of stream");
            let e0 = first_element.inner.paren().expect("first group is paren");
            assert_eq!(e0[0].inner.atom().and_then(|a| a.ident()), Some("define"));
            assert_eq!(e0[1].inner.atom().and_then(|a| a.ident()), Some("x"));
            assert_eq!(
                e0[2]
                    .inner
                    .atom()
                    .and_then(|a| a.number())
                    .and_then(|n| n.to_u64().ok()),
                Some(1)
            )
        }
        {
            let second_element = parser
                .next()
                .expect("parser error")
                .expect("not end of stream");
            let e0 = second_element.inner.paren().expect("second group is paren");
            assert_eq!(e0[0].inner.atom().and_then(|a| a.ident()), Some("define"));
            assert_eq!(e0[1].inner.atom().and_then(|a| a.ident()), Some("y"));
            assert_eq!(
                e0[2]
                    .inner
                    .atom()
                    .and_then(|a| a.number())
                    .and_then(|n| n.to_u64().ok()),
                Some(0x01_ab)
            )
        }

        {
            let third_element = parser
                .next()
                .expect("parser error")
                .expect("not end of stream");
            let e0 = third_element.inner.paren().expect("third group is paren");
            assert_eq!(e0[0].inner.atom().and_then(|a| a.ident()), Some("if"));
            let conditional = e0[1].inner.paren().expect("conditional");
            assert_eq!(
                conditional[0].inner.atom().and_then(|a| a.ident()),
                Some("zero?")
            );
            assert_eq!(
                conditional[1].inner.atom().and_then(|a| a.ident()),
                Some("x")
            );
            let then_expr = e0[2].inner.paren().expect("then");
            assert_eq!(
                then_expr[0].inner.atom().and_then(|a| a.ident()),
                Some("strip")
            );
            assert_eq!(
                then_expr[1]
                    .inner
                    .atom()
                    .and_then(|a| a.string())
                    .map(|s| s.to_string()),
                Some(" ".to_string())
            );
            assert_eq!(
                then_expr[2]
                    .inner
                    .atom()
                    .and_then(|a| a.string())
                    .map(|s| s.to_string()),
                Some("abc".to_string())
            );

            let _else_expr = e0[3].inner.bracket().expect("else");
        }
    }
}
