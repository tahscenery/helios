#![allow(dead_code)]

use crate::source::TextSpan;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxToken {
    _raw: Rc<RawSyntaxToken>,
    span: TextSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RawSyntaxToken {
    kind: TokenKind,
    text: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TokenKind {
    Identifier,
    Keyword,
    Literal,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum SyntaxTrivia {
    Tab(usize),
    Space(usize),
    LineFeed(usize),
    CarriageReturn(usize),
    CarriageReturnLineFeed(usize),
}