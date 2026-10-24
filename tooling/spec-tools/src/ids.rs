//! New ids are date-based (spec/README.md): the minutes since 2026-01-01T00:00Z, or the next free number up when that
//! id already exists for its letter.

use std::collections::HashSet;

use crate::model::{Letter, SpecSet, parse_interpretations, parse_log};

/// `count` new ids for `letter`, in ascending order: the first is the later of `now_minutes` (the minutes since the
/// epoch now, [`crate::time::minutes`]) and `latest` + 1 (the latest id in use), each moved up past every id that
/// already exists in `specs` or was already given in this call.
#[must_use]
pub fn next(
    letter: Letter,
    now_minutes: u64,
    latest: u64,
    count: usize,
    specs: &[SpecSet],
) -> Vec<u64> {
    let mut taken = existing(letter, specs);
    let mut ids = Vec::with_capacity(count);
    let mut id = now_minutes.max(latest.saturating_add(1));
    while ids.len() < count {
        while taken.contains(&id) {
            id += 1;
        }
        ids.push(id);
        taken.insert(id);
    }
    ids
}

/// The latest id in use as given on the command line, with or without its letter (418050, L418050); `None` for
/// anything else, a different letter included.
#[must_use]
pub fn parse_latest(text: &str, letter: Letter) -> Option<u64> {
    let digits = text.strip_prefix(letter.as_char()).unwrap_or(text);
    (!digits.is_empty() && digits.len() <= 18 && digits.bytes().all(|b| b.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
}

/// The L ids of the log headings, or the J ids of the interpretation rows, in every set given (malformed lines are
/// ignored here: spec-check reports them).
#[must_use]
pub fn existing(letter: Letter, specs: &[SpecSet]) -> HashSet<u64> {
    let mut ignored = Vec::new();
    let mut ids = HashSet::new();
    for spec in specs {
        match letter {
            Letter::L => {
                for log in &spec.logs {
                    ids.extend(parse_log(log, &mut ignored).iter().map(|e| e.id));
                }
            }
            Letter::J => {
                if let Some(file) = &spec.interpretations {
                    ids.extend(
                        parse_interpretations(file, &mut ignored)
                            .iter()
                            .map(|r| r.id),
                    );
                }
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SpecFile;
    use crate::time::{EPOCH_UNIX, days_from_civil, minutes};

    /// 2026-10-18T07:30Z: 290 days after 2026-01-01.
    const AT: u64 = 290 * 1440 + 7 * 60 + 30;

    fn log(name: &str, ids: &[u64]) -> SpecFile {
        let entries = ids
            .iter()
            .map(|id| format!("### L{id} · 2026-10-18 · t (decision)\n\n```text\nx\n```\n\n"))
            .collect::<Vec<_>>()
            .concat();
        SpecFile::new(format!("spec/log/{name}"), format!("# Log\n\n{entries}"))
    }

    fn rows(ids: &[u64]) -> SpecFile {
        let rows = ids
            .iter()
            .map(|id| format!("| J{id} | L1 | x |\n"))
            .collect::<Vec<_>>()
            .concat();
        SpecFile::new(
            "spec/interpretations.md",
            format!("| # | Source | Interpretation |\n|---|---|---|\n{rows}"),
        )
    }

    fn set(logs: Vec<SpecFile>, interpretations: Option<SpecFile>) -> SpecSet {
        SpecSet {
            logs,
            interpretations,
            current: Vec::new(),
        }
    }

    fn earlier() -> SpecSet {
        set(
            vec![log("2026-09.md", &[380_000, 380_001])],
            Some(rows(&[380_000])),
        )
    }

    #[test]
    fn the_id_is_the_minutes_since_the_epoch() {
        let at = days_from_civil(2026, 10, 18) * 86_400 + 7 * 3600 + 30 * 60;
        assert!(at > EPOCH_UNIX);
        assert_eq!(minutes(at), Some(AT));
        assert_eq!(next(Letter::L, AT, 0, 1, &[]), [AT]);
    }

    #[test]
    fn now_later_than_the_latest_gives_now() {
        assert_eq!(next(Letter::L, AT, 380_001, 1, &[earlier()]), [AT]);
        assert_eq!(next(Letter::J, AT, AT - 1, 1, &[earlier()]), [AT]);
    }

    #[test]
    fn now_not_later_than_the_latest_continues_after_it() {
        assert_eq!(next(Letter::L, AT, AT, 1, &[earlier()]), [AT + 1]);
        assert_eq!(next(Letter::L, AT, AT + 20, 1, &[earlier()]), [AT + 21]);
    }

    #[test]
    fn a_taken_id_moves_to_the_next_free_number_for_its_letter() {
        let spec = set(vec![log("2026-10.md", &[AT, AT + 1])], Some(rows(&[AT])));
        assert_eq!(
            next(Letter::L, AT, 0, 1, std::slice::from_ref(&spec)),
            [AT + 2]
        );
        assert_eq!(next(Letter::J, AT, 0, 1, &[spec]), [AT + 1]);
    }

    #[test]
    fn several_ids_are_consecutive_skipping_taken_ones() {
        assert_eq!(
            next(Letter::L, AT, 0, 3, &[earlier()]),
            [AT, AT + 1, AT + 2]
        );
        let spec = set(vec![log("2026-10.md", &[AT + 6, AT + 8])], None);
        assert_eq!(
            next(Letter::L, AT, AT + 4, 3, &[spec]),
            [AT + 5, AT + 7, AT + 9]
        );
    }

    #[test]
    fn ids_in_any_given_set_count() {
        let here = set(vec![log("2026-10.md", &[AT])], None);
        let main = set(vec![log("2026-10.md", &[AT + 1])], None);
        assert_eq!(next(Letter::L, AT, 0, 1, &[here, main]), [AT + 2]);
    }

    #[test]
    fn the_latest_is_read_with_or_without_its_letter() {
        assert_eq!(parse_latest("418050", Letter::L), Some(418_050));
        assert_eq!(parse_latest("L418050", Letter::L), Some(418_050));
        assert_eq!(parse_latest("J218", Letter::J), Some(218));
        assert_eq!(parse_latest("0", Letter::L), Some(0));
    }

    #[test]
    fn anything_else_is_refused() {
        for text in [
            "J418050",
            "L",
            "",
            "-5",
            "+5",
            "2026-10-18T07:30Z",
            "99999999999999999999",
            "４",
        ] {
            assert_eq!(parse_latest(text, Letter::L), None, "{text}");
        }
    }
}
