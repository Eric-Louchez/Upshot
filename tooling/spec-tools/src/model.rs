//! The spec files the tools read, and what they read out of them: log entries, interpretation rows, cited ids
//! (formats: spec/README.md).

use std::fmt;

use crate::time::Date;

/// The letter of an id: a log entry (L) or an interpretation row (J).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Letter {
    L,
    J,
}

impl Letter {
    /// `L` or `J`; `None` for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "L" => Some(Self::L),
            "J" => Some(Self::J),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_char(self) -> char {
        match self {
            Self::L => 'L',
            Self::J => 'J',
        }
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

/// A spec file: its path relative to the repository ('/' separators, e.g. `spec/log/2026-10.md`, or
/// `origin/main:spec/log/2026-10.md` when read from Git) and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecFile {
    pub path: String,
    pub text: String,
}

impl SpecFile {
    #[must_use]
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    /// The file name: the path after its last '/'.
    #[must_use]
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
}

/// The files the tools read: the log files, the interpretations, the curated current files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpecSet {
    pub logs: Vec<SpecFile>,
    pub interpretations: Option<SpecFile>,
    pub current: Vec<SpecFile>,
}

/// A log entry: `### L<id> · <date> · <title> (<kind>)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub id: u64,
    pub file: String,
    pub line: usize,
    pub date: Date,
}

/// A row of the interpretations table: `| J<id> | <sources> | <text> |`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpretation {
    pub id: u64,
    pub file: String,
    pub line: usize,
    pub sources: String,
}

/// A problem spec-check reports; line 0 means the whole file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl Problem {
    #[must_use]
    pub fn new(file: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(f, "{}:{}: {}", self.file, self.line, self.message)
        } else {
            write!(f, "{}: {}", self.file, self.message)
        }
    }
}

/// The lines of a text, numbered from 1, without their trailing carriage returns.
pub fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.split('\n')
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim_end_matches('\r')))
}

/// The leading `[1-9][0-9]*` of a text: the digits and the rest; `None` when it doesn't start with one.
fn id_digits(text: &str) -> Option<(&str, &str)> {
    if !text.starts_with(|c: char| matches!(c, '1'..='9')) {
        return None;
    }
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    Some(text.split_at(end))
}

/// Why a heading line is not an entry heading.
enum HeadingError {
    Shape,
    Date(String),
    IdRange,
}

/// `### L<id> · <yyyy-mm-dd> · <title> (<kind>)`: the title starts with a non-space character, the kind holds no
/// parentheses and at least one non-space character.
fn entry_heading(line: &str) -> Result<(u64, Date), HeadingError> {
    let rest = line.strip_prefix("### L").ok_or(HeadingError::Shape)?;
    let (digits, rest) = id_digits(rest).ok_or(HeadingError::Shape)?;
    let rest = rest.strip_prefix(" · ").ok_or(HeadingError::Shape)?;
    let date_text = rest.get(..10).ok_or(HeadingError::Shape)?;
    let shaped = date_text.bytes().enumerate().all(|(i, b)| {
        if i == 4 || i == 7 {
            b == b'-'
        } else {
            b.is_ascii_digit()
        }
    });
    if !shaped {
        return Err(HeadingError::Shape);
    }
    let rest = rest[10..].strip_prefix(" · ").ok_or(HeadingError::Shape)?;
    let inner = rest.strip_suffix(')').ok_or(HeadingError::Shape)?;
    let open = inner.rfind('(').ok_or(HeadingError::Shape)?;
    let kind = &inner[open + 1..];
    let title = inner[..open].strip_suffix(' ').ok_or(HeadingError::Shape)?;
    if kind.contains(')') || kind.trim().is_empty() {
        return Err(HeadingError::Shape);
    }
    if !title.starts_with(|c: char| !c.is_whitespace()) {
        return Err(HeadingError::Shape);
    }
    let date = Date::parse_iso(date_text).ok_or_else(|| HeadingError::Date(date_text.into()))?;
    let id = digits.parse().map_err(|_| HeadingError::IdRange)?;
    Ok((id, date))
}

/// The entries of one log file. Outside fenced blocks every heading but the title on line 1 must be a well-formed
/// entry heading; anything else is a problem.
pub fn parse_log(file: &SpecFile, problems: &mut Vec<Problem>) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let mut fence_line = 0;
    for (n, line) in lines(&file.text) {
        if line.starts_with("```") {
            fence_line = if fence_line == 0 { n } else { 0 };
            continue;
        }
        if fence_line != 0 || !line.starts_with('#') {
            continue;
        }
        if n == 1 && line.starts_with("# ") {
            continue;
        }
        match entry_heading(line) {
            Ok((id, date)) => entries.push(LogEntry {
                id,
                file: file.path.clone(),
                line: n,
                date,
            }),
            Err(HeadingError::Shape) => problems.push(Problem::new(
                &file.path,
                n,
                format!(
                    "malformed entry heading (expected \"### L<id> · <yyyy-mm-dd> · <title> (<kind>)\"): {line}"
                ),
            )),
            Err(HeadingError::Date(date)) => problems.push(Problem::new(
                &file.path,
                n,
                format!("malformed entry heading (no such date {date}): {line}"),
            )),
            Err(HeadingError::IdRange) => problems.push(Problem::new(
                &file.path,
                n,
                format!("malformed entry heading (id out of range): {line}"),
            )),
        }
    }
    if fence_line != 0 {
        problems.push(Problem::new(
            &file.path,
            fence_line,
            "fenced block not closed",
        ));
    }
    entries
}

/// `| J<id> |<sources>|`: the id and the sources column.
fn interpretation_row(line: &str) -> Option<(u64, &str)> {
    let rest = line.strip_prefix("| J")?;
    let (digits, rest) = id_digits(rest)?;
    let rest = rest.strip_prefix(" |")?;
    let end = rest.find('|')?;
    Some((digits.parse().ok()?, &rest[..end]))
}

/// The J rows of the interpretations table; a line that starts like a J row but isn't one is a problem.
pub fn parse_interpretations(file: &SpecFile, problems: &mut Vec<Problem>) -> Vec<Interpretation> {
    let mut rows = Vec::new();
    for (n, line) in lines(&file.text) {
        if !line.starts_with("| J") {
            continue;
        }
        match interpretation_row(line) {
            Some((id, sources)) => rows.push(Interpretation {
                id,
                file: file.path.clone(),
                line: n,
                sources: sources.trim().to_owned(),
            }),
            None => problems.push(Problem::new(
                &file.path,
                n,
                format!(
                    "malformed interpretation row (expected \"| J<id> | <sources> | <text> |\"): {}",
                    shorten(line)
                ),
            )),
        }
    }
    rows
}

/// The L and J ids cited in a text, in order. An id is not part of a longer word (`de-L1`, `XL2`), and `L0` is no id.
#[must_use]
pub fn cited_ids(text: &str) -> Vec<(Letter, u64)> {
    let mut ids = Vec::new();
    let mut previous: Option<char> = None;
    for (i, c) in text.char_indices() {
        let starts_word =
            !previous.is_some_and(|p| p.is_ascii_alphanumeric() || p == '_' || p == '-');
        previous = Some(c);
        let letter = match c {
            'L' => Letter::L,
            'J' => Letter::J,
            _ => continue,
        };
        if !starts_word {
            continue;
        }
        if let Some((digits, _)) = id_digits(&text[i + 1..]) {
            // An id too large for a number is skipped, as no id.
            if let Ok(id) = digits.parse() {
                ids.push((letter, id));
            }
        }
    }
    ids
}

/// The `Source:` lines of a current file (also `Sources:`, bold or in a list item or quote).
pub fn source_lines(file: &SpecFile) -> impl Iterator<Item = (usize, &str)> {
    lines(&file.text).filter(|(_, l)| {
        let rest =
            l.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '>' | '*' | '_' | '-'));
        rest.starts_with("Source:") || rest.starts_with("Sources:")
    })
}

/// A line cut to 120 characters for a message.
#[must_use]
pub fn shorten(line: &str) -> String {
    match line.char_indices().nth(120) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cited_ids_skip_longer_words_and_l0() {
        let ids = cited_ids("L2.4, L1 (3) (refines J1; de-L1, L0 layer, XL5, L7x, J12)");
        assert_eq!(
            ids,
            [
                (Letter::L, 2),
                (Letter::L, 1),
                (Letter::J, 1),
                (Letter::L, 7),
                (Letter::J, 12)
            ]
        );
        assert!(cited_ids("L99999999999999999999999").is_empty());
    }

    #[test]
    fn a_well_formed_heading_parses() {
        let file = SpecFile::new(
            "spec/log/2026-10.md",
            "# Log\n\n### L418027 · 2026-10-18 · a (title) with (parens) (decision, kind)\n",
        );
        let mut problems = Vec::new();
        let entries = parse_log(&file, &mut problems);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 418_027);
        assert_eq!(entries[0].line, 3);
        assert_eq!(entries[0].date.to_string(), "2026-10-18");
    }

    #[test]
    fn source_lines_take_the_markdown_prefixes() {
        let file = SpecFile::new(
            "spec/current/x.md",
            "Source: L1\n- **Sources:** L2\n> _Source:_ L3\nText Source: L4\nSourced: L5\r\n",
        );
        let found: Vec<usize> = source_lines(&file).map(|(n, _)| n).collect();
        assert_eq!(found, [1, 2, 3]);
    }

    #[test]
    fn long_lines_are_shortened() {
        let long = "é".repeat(130);
        assert_eq!(shorten(&long).chars().count(), 121);
        assert_eq!(shorten("short"), "short");
    }
}
