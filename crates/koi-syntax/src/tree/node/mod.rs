#![allow(dead_code)]
pub mod decl;
pub mod expr;

use crate::source::TextSpan;
pub use decl::*;
pub use expr::*;

#[derive(Debug, Eq, PartialEq)]
pub struct SyntaxTree(pub(crate) Vec<Node>);

impl SyntaxTree {
    pub fn children(&self) -> &Vec<Node> {
        &self.0
    }
}

#[derive(Debug, Eq)]
pub enum Node {
    DeclarationNode(Box<dyn DeclarationNode>),
    ExpressionNode(Box<dyn ExpressionNode>),
}

impl Node {
    pub fn span(&self) -> TextSpan {
        #[allow(unreachable_patterns)]
        match self {
            Self::DeclarationNode(node) => node.span(),
            Self::ExpressionNode(node) => node.span(),
            _ => TextSpan::default(),
        }
    }

    pub fn full_span(&self) -> TextSpan {
        #[allow(unreachable_patterns)]
        match self {
            Self::DeclarationNode(node) => node.full_span(),
            Self::ExpressionNode(node) => node.full_span(),
            _ => TextSpan::default(),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        #[allow(unreachable_patterns)]
        match (self, other) {
            (Node::DeclarationNode(lhs), Node::DeclarationNode(rhs)) => lhs == rhs,
            (Node::ExpressionNode(lhs), Node::ExpressionNode(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}