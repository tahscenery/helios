use crate::lexer::{Lexer, LexerMode};
use crate::node::*;
use crate::source::Span;
use crate::token::*;

pub use crate::node::Ast;

pub struct Parser {
    lexer: Lexer,
    peeked_token: Option<Token>,
}

impl Parser {
    pub fn with(lexer: Lexer) -> Self {
        Self { lexer, peeked_token: None }
    }

    pub fn parse(&mut self) -> Ast {
        let mut nodes = Vec::new();

        while !self.lexer.is_at_end() {
            nodes.push(self.parse_program());
        }

        Ast(nodes)
    }
}

impl Parser {
    /// Peeks the next token without consuming it.
    fn peek(&mut self) -> Option<Token> {
        if self.peeked_token.is_none() {
            self.peeked_token = Some(self.lexer.next_token());
        }

        self.peeked_token.clone()
    }

    /// Retrieves the next token.
    fn next_token(&mut self) -> Token {
        match self.peeked_token.take() {
            Some(token) => token,
            None => self.lexer.next_token()
        }
    }

    /// Checks if the next token is of the expected `TokenKind`.
    fn check(&mut self, kind: TokenKind) -> bool {
        match self.peek() {
            Some(token) => token.kind == kind,
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
    fn consume(&mut self, kind: TokenKind) -> Token {
        if let Some(token) = self.peek() {
            if token.kind == kind {
                return self.next_token();
            } else {
                return Token::with(
                    TokenKind::Missing(Box::new(kind)),
                    Span::zero_width(token.span.start)
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
        self.consume_optional(TokenKind::Newline);

        if self.consume_optional(TokenKind::Eof) {
            return Node::Eof;
        }

        Node::ExpressionNode(self.parse_expression())
    }

    fn parse_expression(&mut self) -> Box<dyn ExpressionNode> {
        if
            self.consume_optional(TokenKind::Eof)
                || self.consume_optional(TokenKind::Newline)
        {
            return Box::new(MissingExpressionNode {
                position: self.lexer.current_pos(),
            });
        }

        if self.check(TokenKind::End) {
            return Box::new(SkippedTokenNode { token: self.next_token() });
        }

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
            expression: self.parse_expression_block(),
        })
    }

    fn parse_if_expression(&mut self) -> Box<dyn ExpressionNode> {
        Box::new(IfExpressionNode {
            if_keyword: self.next_token(),
            condition: self.parse_expression(),
            // then_keyword: self.consume(TokenKind::Keyword(Keyword::Then)),
            expression: self.parse_expression_block(),
            else_clause: {
                // self.consume_optional(TokenKind::Newline);
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
            expression: self.parse_expression_block(),
        }
    }

    fn parse_expression_block(&mut self) -> Box<dyn ExpressionNode> {
        self.lexer.push_mode(LexerMode::IndentedBlock);
        let expr_block = self.parse_expression_block_list();
        self.lexer.pop_mode();

        expr_block
    }

    fn parse_expression_block_list(&mut self) -> Box<dyn ExpressionNode> {
        let begin_token = self.next_token();
        Box::new(BlockExpressionNode {
            begin_token: begin_token.clone(),
            expression_list: {
                let mut expressions = Vec::new();
                self.lexer.pop_mode();
                expressions.push(self.parse_expression());

                while self.consume_optional(TokenKind::Newline) {
                    expressions.push(self.parse_expression());
                }

                expressions
            },
            end_token: {
                match self.peek() {
                    Some(Token { kind: TokenKind::End, .. }) => {
                        self.next_token()
                    },
                    Some(token) => {
                        Token::with(
                            TokenKind::End,
                            Span::zero_width(token.span.start)
                        )
                    },
                    None => {
                        Token::with(
                            TokenKind::End,
                            Span::zero_width(self.lexer.current_pos())
                        )
                    }
                }
            },
        })
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Box<dyn ExpressionNode> {
        if self.check(TokenKind::End) {
            return Box::new(MissingExpressionNode { position: self.lexer.current_pos() });
        }

        if self.check(TokenKind::Begin) {
            return self.parse_expression_block_list();
        }

        let mut lhs = self.parse_unary_expression();

        loop {
            let operator = match self.peek() {
                Some(Token { kind: TokenKind::Symbol(symbol), .. }) => symbol,
                _ => break
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
        if let Some(Token { kind: TokenKind::Symbol(symbol), .. }) = self.peek() {
            let token = self.next_token();

            if let Some(right_precedence) = prefix_binding_power(symbol) {
                return Box::new(UnaryExpressionNode {
                    operator: token,
                    operand: self.parse_binary_expression(right_precedence),
                });
            }

            return Box::new(UnexpectedTokenNode { token: self.lexer.next_token() });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Box<dyn ExpressionNode> {
        let token = self.next_token();
        match &token.kind {
            TokenKind::Identifier => {
                Box::new(IdentifierExpressionNode { identifier: token })
            },
            TokenKind::Keyword(Keyword::Unimplemented) => {
                Box::new(UnimplementedExpressionNode { token })
            },
            // TokenKind::Keyword(Keyword::False)
            // | TokenKind::Keyword(Keyword::True)
            TokenKind::Literal(_) => {
                Box::new(LiteralExpressionNode { literal: token })
            },
            TokenKind::GroupingStart(delimiter) => {
                self.lexer.push_mode(LexerMode::Grouping);

                let grouped_expression = GroupedExpressionNode {
                    start_delimiter: token.clone(),
                    expression: self.parse_expression(),
                    end_delimiter: self.consume(TokenKind::GroupingEnd(delimiter.clone())),
                };

                self.lexer.pop_mode();
                Box::new(grouped_expression)
            },
            TokenKind::Newline | TokenKind::Eof => {
                Box::new(MissingExpressionNode { position: self.lexer.current_pos() })
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