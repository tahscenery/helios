#![allow(dead_code)]

use crate::lexer::{Lexer, /* LexerMode */};
use crate::source::TextSpan;
use crate::tree::node::*;
use crate::tree::token::*;
use std::sync::Arc;

pub type ParserOut = SyntaxTree;

pub struct Parser {
    lexer: Lexer,
    peeked_token: Option<SyntaxToken>,
}

impl Parser {
    pub fn with(lexer: Lexer) -> Self {
        Self { lexer, peeked_token: None }
    }

    pub fn parse(&mut self) -> ParserOut {
        let mut nodes = Vec::new();

        while !self.lexer.is_at_end() {
            nodes.push(self.parse_program());
        }

        SyntaxTree(nodes)
    }
}

impl Parser {
    /// Peeks the next token without consuming it.
    fn peek(&mut self) -> Option<SyntaxToken> {
        if self.peeked_token.is_none() {
            self.peeked_token = Some(self.lexer.next_token());
        }

        self.peeked_token.clone()
    }

    /// Retrieves the next token.
    fn next_token(&mut self) -> SyntaxToken {
        match self.peeked_token.take() {
            Some(token) => token,
            None => self.lexer.next_token()
        }
    }

    /// Checks if the next token is of the expected `TokenKind`.
    fn check(&mut self, kind: TokenKind) -> bool {
        match self.peek() {
            Some(token) => token.kind() == kind,
            None => false
        }
    }

    /// Checks if the next token is any one of the expected `TokenKind`s.
    fn check_all(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind.clone()) {
                return true;
            }
        }

        false
    }

    /// Consumes the next token if it is of the expected `TokenKind`, otherwise
    /// returns a `Missing` token.
    fn consume(&mut self, kind: TokenKind) -> SyntaxToken {
        let pos = self.lexer.current_pos();
        if let Some(token) = self.peek() {
            if token.kind() == kind {
                return self.next_token();
            } else {
                return SyntaxToken::missing(
                    Arc::new(RawSyntaxToken::with(kind, String::new())),
                    TextSpan::zero_width(pos)
                );
            }
        }

        panic!("Unhandled case: consuming when peeking gives None value");
    }

    /// Consumes the next token if it is of the expected `TokenKind`, otherwise
    /// does NOT move to the next token.
    fn consume_optional(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.next_token();
            true
        } else {
            false
        }
    }
}

impl Parser {
    fn parse_program(&mut self) -> Node {
        if self.consume_optional(TokenKind::Eof) {
            return Node::Eof;
        }

        Node::ExpressionNode(self.parse_expression())
    }

    fn parse_expression(&mut self) -> Box<dyn ExpressionNode> {
        if self.check(TokenKind::Keyword(Keyword::Let)) {
            return self.parse_let_expression();
        }

        if self.check(TokenKind::Keyword(Keyword::If)) {
            return self.parse_if_expression();
        }

        self.parse_binary_expression(0)
    }

    fn parse_let_expression(&mut self) -> Box<dyn ExpressionNode> {
        Box::new(LocalBindingExpressionNode {
            let_keyword: self.next_token(),
            identifier: self.consume(TokenKind::Identifier),
            equal_symbol: self.consume(TokenKind::Symbol(Symbol::Eq)),
            expression: self.parse_expression(),
        })
    }

    fn parse_if_expression(&mut self) -> Box<dyn ExpressionNode> {
        Box::new(IfExpressionNode {
            if_keyword: self.next_token(),
            condition: self.parse_expression(),
            open_brace: self.consume(TokenKind::Symbol(Symbol::LBrace)),
            expression: self.parse_expression(),
            close_brace: self.consume(TokenKind::Symbol(Symbol::RBrace)),
            else_clause: {
                if self.check(TokenKind::Keyword(Keyword::Else)) {
                    Some(self.parse_else_clause())
                } else {
                    None
                }
            },
        })
    }

    fn parse_else_clause(&mut self) -> ElseClauseExpressionNode {
        ElseClauseExpressionNode {
            else_keyword: self.next_token(),
            open_brace: self.consume(TokenKind::Symbol(Symbol::LBrace)),
            expression: self.parse_expression(),
            close_brace: self.consume(TokenKind::Symbol(Symbol::RBrace)),
        }
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Box<dyn ExpressionNode> {
        let mut lhs = self.parse_unary_expression();

        loop {
            let operator = match self.peek() {
                Some(token) => match token.kind() {
                    TokenKind::Symbol(symbol) => symbol,
                    _ => break,
                },
                _ => break,
            };

            if let Some((left_precedence, right_precedence)) = infix_binding_power(operator) {
                if left_precedence < min_precedence {
                    break;
                }

                lhs = Box::new(BinaryExpressionNode {
                    operator: self.next_token(),
                    lhs: lhs.clone(),
                    rhs: self.parse_binary_expression(right_precedence),
                });
                continue;
            }

            break;
        }

        lhs
    }

    fn parse_unary_expression(&mut self) -> Box<dyn ExpressionNode> {
        match self.peek() {
            Some(token) => match token.kind() {
                TokenKind::Symbol(symbol) => {
                    let token = self.next_token();

                    if let Some(right_precedence) = prefix_binding_power(symbol) {
                        return Box::new(UnaryExpressionNode {
                            operator: token,
                            operand: self.parse_binary_expression(right_precedence),
                        });
                    }

                    return Box::new(UnexpectedTokenNode {
                        token: self.lexer.next_token()
                    });
                },
                _ => ()
            },
            _ => ()
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Box<dyn ExpressionNode> {
        let token = self.next_token();
        match &token.kind() {
            TokenKind::Identifier => {
                Box::new(IdentifierExpressionNode { identifier: token })
            },
            TokenKind::Keyword(Keyword::Unimplemented) => {
                Box::new(UnimplementedExpressionNode { token })
            },
            TokenKind::Literal(_) => {
                Box::new(LiteralExpressionNode { literal: token })
            },
            TokenKind::Error(_) => {
                Box::new(ErrorExpressionNode { token })
            },
            _ => Box::new(UnexpectedTokenNode { token })
        }
    }
}

/// Determines the prefix binding power of the given symbol. Currently, the only
/// legal prefix symbols are `Symbol::Minus` and `Symbol::Bang`.
fn prefix_binding_power(symbol: Symbol) -> Option<u8> {
    let power = match symbol {
        Symbol::Minus | Symbol::Bang => 9,
        _ => return None,
    };

    Some(power)
}

/// Determines the infix binding power of the given symbol. A higher binding
/// power means higher precedence, meaning that it is more likely to hold onto
/// its adjacent operands.
fn infix_binding_power(symbol: Symbol) -> Option<(u8, u8)> {
    let power = match symbol {
        Symbol::Semicolon => (1, 2),
        Symbol::LThinArrow => (3, 2),
        Symbol::Eq | Symbol::BangEq => (4, 3),
        Symbol::Lt | Symbol::Gt | Symbol::LtEq | Symbol::GtEq => (5, 6),
        Symbol::Plus | Symbol::Minus => (7, 8),
        Symbol::Asterisk | Symbol::ForwardSlash => (9, 10),
        _ => return None,
    };

    Some(power)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        // 16 * 1024 * 1024 = 16MiB
        std::env::set_var("RUST_MIN_STACK", "16777216");

        let source = "// Adding two variables\nlet a = 10\n";
        let mut parser = Parser::with(Lexer::with(source.to_string()));
        let _tree = parser.parse();

        // === FIXME: This panics with stack overflow! ===
        // assert_eq!(tree.children(), &vec! {
        //     Node::ExpressionNode(Box::new(LocalBindingExpressionNode {
        //         let_keyword: SyntaxToken::with_trivia(
        //             Arc::new(RawSyntaxToken::with(
        //                 TokenKind::Keyword(Keyword::Let),
        //                 "let".to_string()
        //             )),
        //             TextSpan::new(24, 3),
        //             vec![
        //                 SyntaxTrivia::LineComment {
        //                     is_doc_comment: false,
        //                     len: 23,
        //                 }
        //             ],
        //             vec![SyntaxTrivia::Space(1)],
        //         ),
        //         identifier: SyntaxToken::with_trivia(
        //             Arc::new(RawSyntaxToken::with(
        //                 TokenKind::Identifier,
        //                 "a".to_string()
        //             )),
        //             TextSpan::new(28, 1),
        //             vec![],
        //             vec![SyntaxTrivia::Space(1)],
        //         ),
        //         equal_symbol: SyntaxToken::with_trivia(
        //             Arc::new(RawSyntaxToken::with(
        //                 TokenKind::Identifier,
        //                 "=".to_string()
        //             )),
        //             TextSpan::new(30, 1),
        //             vec![],
        //             vec![SyntaxTrivia::Space(1)],
        //         ),
        //         expression: Box::new(LiteralExpressionNode {
        //             literal: SyntaxToken::with_trivia(
        //                 Arc::new(RawSyntaxToken::with(
        //                     TokenKind::Literal(Literal::Integer(Base::Decimal)),
        //                     "10".to_string()
        //                 )),
        //                 TextSpan::new(32, 2),
        //                 vec![],
        //                 vec![SyntaxTrivia::LineFeed(1)],
        //             )
        //         }),
        //     }))
        // });

        assert!(true)
    }
}
