//! tooling/up spec-check: the rules every spec file keeps (spec/README.md).

use std::collections::{HashMap, HashSet};

use crate::model::{
    Letter, LogEntry, Problem, SpecFile, SpecSet, cited_ids, parse_interpretations, parse_log,
    source_lines,
};
use crate::time::time_of;

/// The problems found (none: the spec passes) and the counts to print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub problems: Vec<Problem>,
    pub summary: Vec<String>,
}

impl CheckResult {
    #[must_use]
    pub fn ok(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Checks a spec: the log's headings, order and placement, ids defined twice, and cited ids that don't exist.
#[must_use]
pub fn check(spec: &SpecSet) -> CheckResult {
    let mut problems = Vec::new();
    let mut summary = Vec::new();

    // The log: well-formed headings, ascending in each file, each id in its file's month, none twice.
    let mut logs: Vec<&SpecFile> = spec.logs.iter().collect();
    logs.sort_by(|a, b| a.path.cmp(&b.path));
    let mut entries: Vec<LogEntry> = Vec::new();
    let mut per_file = Vec::new();
    for file in logs {
        let file_entries = parse_log(file, &mut problems);
        for pair in file_entries.windows(2) {
            if pair[1].id < pair[0].id {
                problems.push(Problem::new(
                    &file.path,
                    pair[1].line,
                    format!(
                        "L{} after L{} (line {}): the log is in ascending id order",
                        pair[1].id, pair[0].id, pair[0].line
                    ),
                ));
            }
        }
        check_placement(file, &file_entries, &mut problems);
        per_file.push(format!("{} {}", file.name(), file_entries.len()));
        entries.extend(file_entries);
    }
    duplicates(
        Letter::L,
        entries.iter().map(|e| (e.id, e.file.as_str(), e.line)),
        &mut problems,
    );
    summary.push(if spec.logs.is_empty() {
        "log: no files".to_owned()
    } else {
        format!(
            "log: {} entries{} in {} files ({})",
            entries.len(),
            range(Letter::L, entries.iter().map(|e| e.id)),
            spec.logs.len(),
            per_file.join(", ")
        )
    });

    // The interpretations: well-formed rows, none twice, every L and J they cite exists.
    let rows = spec
        .interpretations
        .as_ref()
        .map(|f| parse_interpretations(f, &mut problems))
        .unwrap_or_default();
    duplicates(
        Letter::J,
        rows.iter().map(|r| (r.id, r.file.as_str(), r.line)),
        &mut problems,
    );
    let known_l: HashSet<u64> = entries.iter().map(|e| e.id).collect();
    let known_j: HashSet<u64> = rows.iter().map(|r| r.id).collect();
    let exists = |letter: Letter, id: u64| match letter {
        Letter::L => known_l.contains(&id),
        Letter::J => known_j.contains(&id),
    };
    let mut cited = 0;
    for row in &rows {
        for (letter, id) in cited_ids(&row.sources) {
            cited += 1;
            if !exists(letter, id) {
                problems.push(Problem::new(
                    &row.file,
                    row.line,
                    format!("J{} cites {letter}{id}, which does not exist", row.id),
                ));
            }
        }
    }
    summary.push(if spec.interpretations.is_none() {
        "interpretations: no file".to_owned()
    } else {
        format!(
            "interpretations: {} rows{}; their sources cite {cited} L/J ids",
            rows.len(),
            range(Letter::J, rows.iter().map(|r| r.id))
        )
    });

    // The curated current files: every id on a Source: line exists.
    summary.push(check_current(&spec.current, &exists, &mut problems));
    CheckResult { problems, summary }
}

/// Every id on a `Source:` line of the current files exists; the summary line.
fn check_current(
    current: &[SpecFile],
    exists: &impl Fn(Letter, u64) -> bool,
    problems: &mut Vec<Problem>,
) -> String {
    if current.is_empty() {
        return "current: no files".to_owned();
    }
    let mut count = 0;
    for file in current {
        for (n, text) in source_lines(file) {
            count += 1;
            for (letter, id) in cited_ids(text) {
                if !exists(letter, id) {
                    problems.push(Problem::new(
                        &file.path,
                        n,
                        format!("Source: cites {letter}{id}, which does not exist"),
                    ));
                }
            }
        }
    }
    format!("current: {} files, {count} Source: lines", current.len())
}

/// `yyyy-mm.md`: the year and month.
fn month_file(name: &str) -> Option<(i64, u32)> {
    let stem = name.strip_suffix(".md")?;
    let b = stem.as_bytes();
    let shaped = b.len() == 7
        && b.iter().enumerate().all(|(i, &c)| {
            if i == 4 {
                c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    shaped.then(|| Some((stem[..4].parse().ok()?, stem[5..].parse().ok()?)))?
}

/// `2026-10.md` holds the date-based ids of October 2026 (UTC); no other file names.
fn check_placement(file: &SpecFile, entries: &[LogEntry], problems: &mut Vec<Problem>) {
    let Some((year, month)) = month_file(file.name()) else {
        problems.push(Problem::new(
            &file.path,
            0,
            "unexpected log file name (<yyyy>-<mm>.md)",
        ));
        return;
    };
    for e in entries {
        let at = time_of(e.id);
        if at.date.year != year || at.date.month != month {
            problems.push(Problem::new(
                &file.path,
                e.line,
                format!(
                    "L{} is a date-based id of {at}Z, not of {year:04}-{month:02}",
                    e.id
                ),
            ));
        }
    }
}

/// An id defined more than once: one problem per id, at its second definition.
fn duplicates<'a>(
    letter: Letter,
    definitions: impl Iterator<Item = (u64, &'a str, usize)>,
    problems: &mut Vec<Problem>,
) {
    let mut groups: Vec<(u64, Vec<(&str, usize)>)> = Vec::new();
    let mut index: HashMap<u64, usize> = HashMap::new();
    for (id, file, line) in definitions {
        let i = *index.entry(id).or_insert_with(|| {
            groups.push((id, Vec::new()));
            groups.len() - 1
        });
        groups[i].1.push((file, line));
    }
    for (id, places) in groups.iter().filter(|(_, p)| p.len() > 1) {
        let list: Vec<String> = places.iter().map(|(f, l)| format!("{f}:{l}")).collect();
        problems.push(Problem::new(
            places[1].0,
            places[1].1,
            format!(
                "{letter}{id} is defined {} times ({})",
                places.len(),
                list.join(", ")
            ),
        ));
    }
}

/// ` (L1–L9)`, or nothing for no ids.
fn range(letter: Letter, ids: impl Iterator<Item = u64>) -> String {
    let (lo, hi) = ids.fold((u64::MAX, 0), |(lo, hi), id| (lo.min(id), hi.max(id)));
    if lo > hi {
        String::new()
    } else {
        format!(" ({letter}{lo}–{letter}{hi})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::from_working_tree;
    use std::path::Path;

    // 418027 minutes after 2026-01-01T00:00Z is 2026-10-18; 380000 is 2026-09-21; 420000 is 2026-10-19.
    const OCT_1: u64 = 418_027;
    const OCT_2: u64 = 418_100;
    const OCT_3: u64 = 420_000;
    const SEP_1: u64 = 380_000;
    const SEP_2: u64 = 380_001;

    fn entry(id: u64) -> String {
        format!(
            "### L{id} · 2026-10-06 · a title (decision)\n\nContext: c.\n\n```text\nmessage\n```\n\n"
        )
    }

    fn log(name: &str, entries: &[String]) -> SpecFile {
        SpecFile::new(
            format!("spec/log/{name}"),
            format!("# Log\n\nHeader.\n\n{}", entries.concat()),
        )
    }

    fn rows(rows: &[&str]) -> SpecFile {
        SpecFile::new(
            "spec/interpretations.md",
            format!(
                "# Interpretations\n\n| # | Source | Interpretation |\n|---|---|---|\n{}\n",
                rows.join("\n")
            ),
        )
    }

    fn valid_logs() -> Vec<SpecFile> {
        vec![
            log("2026-09.md", &[entry(SEP_1), entry(SEP_2)]),
            log("2026-10.md", &[entry(OCT_1)]),
        ]
    }

    fn valid() -> SpecSet {
        SpecSet {
            logs: valid_logs(),
            interpretations: Some(rows(&[
                &format!("| J1 | L{SEP_1} \"x\" | a |"),
                &format!("| J2 | L{SEP_2}.4, L{SEP_1} (3) (refines J1; de-L1, L0 layer) | b |"),
            ])),
            current: Vec::new(),
        }
    }

    fn problems(spec: &SpecSet) -> Vec<String> {
        check(spec)
            .problems
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    fn single(spec: &SpecSet) -> String {
        let p = problems(spec);
        assert_eq!(p.len(), 1, "{p:#?}");
        p.into_iter().next().unwrap_or_default()
    }

    #[test]
    fn a_valid_spec_passes_with_counts() {
        let result = check(&valid());
        assert!(result.ok(), "{:#?}", result.problems);
        assert_eq!(
            result.summary,
            [
                format!(
                    "log: 3 entries (L{SEP_1}–L{OCT_1}) in 2 files (2026-09.md 2, 2026-10.md 1)"
                ),
                "interpretations: 2 rows (J1–J2); their sources cite 4 L/J ids".to_owned(),
                "current: no files".to_owned(),
            ]
        );
    }

    #[test]
    fn an_empty_spec_passes() {
        let result = check(&SpecSet::default());
        assert!(result.ok());
        assert_eq!(
            result.summary,
            [
                "log: no files",
                "interpretations: no file",
                "current: no files"
            ]
        );
    }

    #[test]
    fn an_l_defined_twice_fails() {
        let mut spec = valid();
        spec.logs[1] = log("2026-10.md", &[entry(OCT_1), entry(OCT_1)]);
        let p = single(&spec);
        assert!(p.contains(&format!("L{OCT_1} is defined 2 times")), "{p}");
        assert!(p.starts_with("spec/log/2026-10.md:"), "{p}");
    }

    #[test]
    fn a_j_defined_twice_fails() {
        let mut spec = valid();
        spec.interpretations = Some(rows(&[
            &format!("| J1 | L{SEP_1} | a |"),
            &format!("| J1 | L{SEP_2} | b |"),
        ]));
        assert!(single(&spec).contains(
            "J1 is defined 2 times (spec/interpretations.md:5, spec/interpretations.md:6)"
        ));
    }

    #[test]
    fn a_malformed_heading_fails() {
        for heading in [
            "### L3 · 2026-10-06 · no kind",
            "### L3 - 2026-10-06 - dashes (decision)",
            "### L3 · 2026-02-30 · no such day (decision)",
            "### L3 · 2026-10-06 ·  (decision)",
            "### L3 · 2026-10-06 · empty kind ( )",
            "### L03x · 2026-10-06 · bad id (decision)",
            "### L99999999999999999999999 · 2026-10-06 · huge id (decision)",
            "## L3 · 2026-10-06 · wrong level (decision)",
            "#### Some heading",
        ] {
            let mut spec = valid();
            spec.logs[1] = log("2026-10.md", &[entry(OCT_1), format!("{heading}\n\n")]);
            let p = single(&spec);
            assert!(p.starts_with("spec/log/2026-10.md:"), "{p}");
            assert!(p.contains("malformed entry heading"), "{p}");
        }
    }

    #[test]
    fn headings_inside_a_fenced_block_are_the_message() {
        let mut spec = valid();
        spec.logs[1] = log(
            "2026-10.md",
            &[
                entry(OCT_1),
                format!(
                    "### L{OCT_2} · 2026-10-18 · t (decision)\n\n```text\n### not a heading\n# nor this\n```\n\n"
                ),
            ],
        );
        assert_eq!(problems(&spec), Vec::<String>::new());
    }

    #[test]
    fn an_unclosed_fence_fails() {
        let mut spec = valid();
        spec.logs[1] = log(
            "2026-10.md",
            &[
                entry(OCT_1),
                format!("### L{OCT_2} · 2026-10-18 · t (decision)\n\n```text\nx\n"),
            ],
        );
        assert!(single(&spec).contains("fenced block not closed"));
    }

    #[test]
    fn a_log_out_of_order_fails() {
        let mut spec = valid();
        spec.logs[0] = log("2026-09.md", &[entry(SEP_2), entry(SEP_1)]);
        assert!(single(&spec).contains(&format!("L{SEP_1} after L{SEP_2}")));
    }

    #[test]
    fn an_id_outside_its_files_month_fails() {
        let mut spec = valid();
        spec.logs = vec![
            log("2026-09.md", &[entry(SEP_1), entry(SEP_2)]),
            log(
                "2026-10.md",
                &[entry(SEP_1 + 2), entry(OCT_1), entry(OCT_3)],
            ),
            log("L0001-L0099.md", &[]),
            log("notes.md", &[]),
        ];
        let p = problems(&spec);
        assert_eq!(p.len(), 3, "{p:#?}");
        assert!(p.iter().any(|p| p.contains(&format!(
            "L{} is a date-based id of 2026-09-21 21:22Z, not of 2026-10",
            SEP_1 + 2
        ))));
        assert!(
            p.iter()
                .any(|p| p.starts_with("spec/log/L0001-L0099.md: unexpected log file name"))
        );
        assert!(
            p.iter()
                .any(|p| p.starts_with("spec/log/notes.md: unexpected log file name"))
        );
    }

    #[test]
    fn a_cited_id_that_does_not_exist_fails() {
        let mut spec = valid();
        spec.interpretations = Some(rows(&[&format!(
            "| J1 | L{SEP_1}, L7 (refines J9) | text cites L8 and J10, not checked |"
        )]));
        assert_eq!(
            problems(&spec),
            [
                "spec/interpretations.md:5: J1 cites L7, which does not exist",
                "spec/interpretations.md:5: J1 cites J9, which does not exist",
            ]
        );
    }

    #[test]
    fn a_malformed_row_fails() {
        let mut spec = valid();
        spec.interpretations = Some(rows(&[&format!("| J1 | L{SEP_1} | a |"), "| J2: L1 | b |"]));
        assert!(single(&spec).contains("malformed interpretation row"));
    }

    #[test]
    fn current_source_lines_must_cite_existing_ids() {
        let mut spec = valid();
        spec.current = vec![SpecFile::new(
            "spec/current/reading.md",
            format!(
                "# Reading\n\nSource: L{SEP_1}, J2\n\n- **Sources:** L{OCT_1}, L5\n\nText citing L6 is not a Source: line.\n"
            ),
        )];
        let result = check(&spec);
        assert_eq!(
            result
                .problems
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["spec/current/reading.md:5: Source: cites L5, which does not exist"]
        );
        assert!(
            result
                .summary
                .contains(&"current: 1 files, 2 Source: lines".to_owned())
        );
    }

    /// The repository's own spec passes, once it has one.
    #[test]
    fn the_repositorys_spec_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        if !root.join("spec").is_dir() {
            return;
        }
        let spec = from_working_tree(&root).expect("the spec files are readable");
        let result = check(&spec);
        let list: Vec<String> = result.problems.iter().map(ToString::to_string).collect();
        assert!(result.ok(), "{}", list.join("\n"));
    }
}
