//! `tooling/up spec-id L|J <latest> [<count>] [--no-git]` and `tooling/up spec-check` (spec/README.md).

use std::path::PathBuf;
use std::process::ExitCode;

use upshot_spec_tools::model::Letter;
use upshot_spec_tools::{checker, ids, sources, time};

const USAGE: &str = "\
Usage: tooling/up spec-id L|J <latest> [<count>] [--no-git]
       tooling/up spec-check
spec-id prints <count> (default 1) new ids for log entries (L) or interpretation rows (J), one per line.
<latest> is the latest id already in use (e.g. 418050 or L418050; 0 when there is none). The first id is the
minutes from 2026-01-01T00:00Z to now (UTC), or <latest> + 1 if that is higher; each id takes the next number up
while it exists in spec/log/*.md (L) or spec/interpretations.md (J), here or at origin/main (as last fetched;
--no-git skips that), or was printed before it.
spec-check reads spec/log/, spec/interpretations.md and spec/current/ and fails (exit 1) with the list of
problems: an id defined twice, a malformed log heading, a log file not named <yyyy>-<mm>.md, out of ascending
order or holding an id outside its month, an L or J id cited in the interpretations' sources column (or on a
Source: line of spec/current/*.md) that does not exist. Otherwise it prints the counts (exit 0).";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if matches!(args.as_slice(), ["-h" | "--help"] | [_, "-h" | "--help"]) {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let repository = std::env::var_os("UP_REPOSITORY")
        .filter(|r| !r.is_empty())
        .map_or_else(
            || std::env::current_dir().unwrap_or_default(),
            PathBuf::from,
        );
    if !repository.join("spec").is_dir() {
        eprintln!(
            "spec: no spec/ folder in {} (run through tooling/up)",
            repository.display()
        );
        return ExitCode::from(2);
    }
    match args.as_slice() {
        ["id", letter, latest, rest @ ..] => match Letter::parse(letter) {
            Some(letter) => spec_id(&repository, letter, latest, rest),
            None => usage(),
        },
        ["check"] => spec_check(&repository),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn spec_id(
    repository: &std::path::Path,
    letter: Letter,
    latest_arg: &str,
    rest: &[&str],
) -> ExitCode {
    let no_git = rest.iter().filter(|a| **a == "--no-git").count();
    let others: Vec<&str> = rest.iter().copied().filter(|a| *a != "--no-git").collect();
    if no_git > 1 || others.len() > 1 {
        return usage();
    }
    let Some(latest) = ids::parse_latest(latest_arg, letter) else {
        eprintln!("spec-id: {latest_arg} is not an id number (e.g. 418050 or {letter}418050)");
        return ExitCode::from(2);
    };
    let count = match others.first() {
        None => 1,
        Some(text) => match text.parse::<usize>() {
            Ok(n) if n >= 1 && text.bytes().all(|b| b.is_ascii_digit()) => n,
            _ => {
                eprintln!("spec-id: the count {text} is not a whole number from 1 up");
                return ExitCode::from(2);
            }
        },
    };
    let Some(now) = time::minutes(time::now_unix()) else {
        eprintln!("spec-id: the clock is before 2026-01-01T00:01Z: no date-based id");
        return ExitCode::from(2);
    };
    let mut sets = match sources::from_working_tree(repository) {
        Ok(set) => vec![set],
        Err(e) => {
            eprintln!("spec-id: cannot read the spec files: {e}");
            return ExitCode::from(2);
        }
    };
    if no_git == 0
        && let Some(main) = sources::from_git(repository, "origin/main")
    {
        sets.push(main);
    }
    for id in ids::next(letter, now, latest, count, &sets) {
        println!("{letter}{id}");
    }
    ExitCode::SUCCESS
}

fn spec_check(repository: &std::path::Path) -> ExitCode {
    let spec = match sources::from_working_tree(repository) {
        Ok(spec) => spec,
        Err(e) => {
            eprintln!("spec-check: cannot read the spec files: {e}");
            return ExitCode::from(2);
        }
    };
    let result = checker::check(&spec);
    for line in &result.summary {
        println!("{line}");
    }
    if result.ok() {
        println!("spec-check: ok");
        return ExitCode::SUCCESS;
    }
    eprintln!("spec-check: {} problem(s)", result.problems.len());
    for problem in &result.problems {
        eprintln!("  {problem}");
    }
    ExitCode::from(1)
}
