mod diagnostic;
mod parser;
mod source;

pub use diagnostic::Diagnostic;
pub use source::{Position, Source};
pub use parser::{Ast, token};

type Result<T> = std::result::Result<T, Vec<Diagnostic>>;

pub fn start(_file_name: &str) -> Result<()> {
    unimplemented!()
}

pub fn tokenize<'a>(source: Source<'a>) -> Ast {
    parser::parse(source)
}
