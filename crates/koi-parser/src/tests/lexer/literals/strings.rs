use crate::token::*;
use crate::lexer::*;
use crate::source::Position;
use super::read_from_string;

macro_rules! test_string {
    ($string:expr, $content:expr) => {
        test_string!($string, $content, $string.len())
    };
    ($string:expr, $content:expr, $size:expr) => {
        create_test! {
            $string,
            vec! {
                Token::with(
                    TokenKind::Literal(Literal::Str($content)),
                    Position::new(0, 0)..Position::new(0, $size)
                )
            }
        }
    }
}

#[test]
fn test_string_literals() {
    test_string!(r#""""#, "".to_string());
    test_string!(r#""   ""#, "   ".to_string());
    test_string!(r#""Hello, world!""#, "Hello, world!".to_string());
    test_string!(r#""Hello\nworld!""#, "Hello\nworld!".to_string());
    test_string!(r#""\\\n\t\r""#, "\\\n\t\r".to_string());

    create_test! {
r#""This is the first line. \
This is the second line. \
This is the third line. \
This is the fourth line. \
This is the last line.""#,
        vec! {
            Token::with(
                TokenKind::Literal(Literal::Str(
                    "This is the first line. \
                     This is the second line. \
                     This is the third line. \
                     This is the fourth line. \
                     This is the last line.".to_string(),
                )),
                Position::new(0, 0)..Position::new(4, 23)
            )
        }
    }

    create_test! {
r#""\
    Hello, world! My name is \
    PAL. I am a friendly computer \
    looking for no harm.\
""#,
        vec! {
            Token::with(
                TokenKind::Literal(Literal::Str(
                    "Hello, world! My name is PAL. I am a friendly \
                     computer looking for no harm.".to_string()
                )),
                Position::new(0, 0)..Position::new(4, 1)
            )
        }
    }
}

#[test]
fn test_unterminated_string_literals() {
    create_test! {
        r#""Hello, world!"#,
        vec! {
            Token::with(
                TokenKind::Error(LexerError::UnterminatedStrLiteral),
                Position::new(0, 0)..Position::new(0, 14)
            )
        }
    }

    create_test! {
        r#"r"C:\Documents\Newsletters\Summer2018.pdf"#,
        vec! {
            Token::with(
                TokenKind::Error(LexerError::UnterminatedStrLiteral),
                Position::new(0, 0)..Position::new(0, 41)
            )
        }
    }

    create_test! {
        r#"f"1 + 2 = {1 + 2}"#,
        vec! {
            Token::with(
                TokenKind::Error(LexerError::UnterminatedStrLiteral),
                Position::new(0, 0)..Position::new(0, 17)
            )
        }
    }
}

#[test]
fn test_invalid_string_literals() {
    create_test! {
        r#""a\b\c\de""#,
        vec! {
            Token::with(
                TokenKind::Error(LexerError::UnknownEscapeChar('b')),
                Position::new(0, 0)..Position::new(0, 10)
            )
        }
    }

    create_test! {
        r#""Hello. \World""#,
        vec! {
            Token::with(
                TokenKind::Error(LexerError::UnknownEscapeChar('W')),
                Position::new(0, 0)..Position::new(0, 15)
            )
        }
    }
}

#[test]
fn test_raw_string_literals() {
    test_string! {
        r#"r"C:\Documents\Newsletters\Summer2018.pdf""#,
        "C:\\Documents\\Newsletters\\Summer2018.pdf".to_string()
    }

    test_string! {
        r#"r"\(\*(?!\*[^\)])""#,
        "\\(\\*(?!\\*[^\\)])".to_string()
    }
}

#[test]
fn test_interpolated_string_literals() {
    create_test! {
        r#"f"1 + 2 = {1 + 2}""#,
        vec! {
            Token::with(
                TokenKind::Literal(Literal::FStr("1 + 2 = {1 + 2}".to_string())),
                Position::new(0, 0)..Position::new(0, 18)
            )
        }
    }

    create_test! {
        r#"f"You scored {score} points!""#,
        vec! {
            Token::with(
                TokenKind::Literal(Literal::FStr("You scored {score} points!".to_string())),
                Position::new(0, 0)..Position::new(0, 29)
            )
        }
    }
}