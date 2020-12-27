use colored::*;
use std::fmt::{self, Display};
use std::ops::Range;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Location<FileId> {
    pub file_id: FileId,
    pub range: Range<usize>,
}

impl<FileId> Location<FileId> {
    pub fn new(file_id: FileId, range: impl Into<Range<usize>>) -> Self {
        Self {
            file_id,
            range: range.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Severity {
    Bug = 3,
    Error = 2,
    Warning = 1,
    Note = 0,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Note
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormattedTextSegment {
    LineBreak,
    Text(String),
    Code(String),
    CodeBlock(String),
    List(Vec<FormattedText>),
}

impl FormattedTextSegment {
    pub fn text(text: impl Display) -> Self {
        Self::Text(text.to_string())
    }

    pub fn code(code: impl Display) -> Self {
        Self::Code(code.to_string())
    }

    pub fn code_block(code: impl Display) -> Self {
        Self::CodeBlock(code.to_string())
    }

    pub fn list(list: impl Into<Vec<FormattedText>>) -> Self {
        Self::List(list.into())
    }
}

impl Display for FormattedTextSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let colorize = colored::control::SHOULD_COLORIZE.should_colorize();

        match self {
            Self::LineBreak => write!(f, "\n\n"),
            Self::Text(text) => write!(f, "{}", text),
            Self::Code(code) => {
                if colorize {
                    write!(f, "{}", code.yellow())
                } else {
                    write!(f, "{}", code)
                }
            }
            Self::CodeBlock(block) => {
                if colorize {
                    write!(f, "    {}", block.yellow())
                } else {
                    write!(f, "    {}", block)
                }
            }
            Self::List(lines) => {
                for line in lines {
                    write!(f, "    {}\n", line)?;
                }

                Ok(())
            }
        }
    }
}

impl From<String> for FormattedTextSegment {
    fn from(string: String) -> Self {
        FormattedTextSegment::Text(string)
    }
}

impl From<&str> for FormattedTextSegment {
    fn from(string: &str) -> Self {
        FormattedTextSegment::Text(string.to_string())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FormattedText {
    segments: Vec<FormattedTextSegment>,
}

impl FormattedText {
    pub fn new(segments: impl Into<Vec<FormattedTextSegment>>) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    pub fn text(mut self, text: impl Display) -> Self {
        self.segments.push(FormattedTextSegment::text(text));
        self
    }

    pub fn code(mut self, code: impl Display) -> Self {
        self.segments.push(FormattedTextSegment::code(code));
        self
    }

    pub fn code_block(mut self, code: impl Display) -> Self {
        self.segments.push(FormattedTextSegment::LineBreak);
        self.segments.push(FormattedTextSegment::code_block(code));
        self.segments.push(FormattedTextSegment::LineBreak);
        self
    }

    pub fn list(mut self, list: impl Into<Vec<FormattedText>>) -> Self {
        self.segments.push(FormattedTextSegment::LineBreak);
        self.segments.push(FormattedTextSegment::list(list));
        self.segments.push(FormattedTextSegment::LineBreak);
        self
    }

    pub fn line_break(mut self) -> Self {
        self.segments.push(FormattedTextSegment::LineBreak);
        self
    }
}

impl From<String> for FormattedText {
    fn from(s: String) -> Self {
        Self::new([s.into()])
    }
}

impl From<&str> for FormattedText {
    fn from(s: &str) -> Self {
        Self::new([s.into()])
    }
}

impl Display for FormattedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in &self.segments {
            write!(f, "{}", segment)?;
        }

        Ok(())
    }
}

/// A diagnostic that provides information about a found issue in a Helios
/// source file like errors or warnings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagnostic<FileId> {
    pub location: Location<FileId>,
    pub severity: Severity,
    pub title: String,
    pub description: Option<FormattedText>,
    pub message: FormattedText,
    pub hint: Option<FormattedText>,
}

impl<FileId> Diagnostic<FileId>
where
    FileId: Default,
{
    pub fn new(
        location: Location<FileId>,
        severity: Severity,
        title: impl Into<String>,
        description: impl Into<Option<FormattedText>>,
        message: impl Into<FormattedText>,
        hint: impl Into<Option<FormattedText>>,
    ) -> Self {
        Self {
            location,
            severity,
            title: title.into(),
            description: description.into(),
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub fn bug(title: impl Into<String>) -> Self {
        Self {
            severity: Severity::Bug,
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn error(title: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn warning(title: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn note(title: impl Into<String>) -> Self {
        Self {
            severity: Severity::Note,
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn description(
        mut self,
        description: impl Into<FormattedText>,
    ) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn message(mut self, message: impl Into<FormattedText>) -> Self {
        self.message = message.into();
        self
    }

    pub fn location(mut self, location: Location<FileId>) -> Self {
        self.location = location;
        self
    }

    pub fn hint(mut self, hint: impl Into<FormattedText>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_severity_ok() {
        let mut is_ok = true;

        for severity in &[Severity::Note, Severity::Note, Severity::Warning] {
            is_ok &= *severity < Severity::Error;
        }

        assert!(is_ok);
    }

    #[test]
    fn test_compare_severity_not_ok() {
        let mut is_ok = true;

        for severity in &[Severity::Note, Severity::Error, Severity::Warning] {
            is_ok &= *severity < Severity::Error;
        }

        assert!(!is_ok);
    }
}
