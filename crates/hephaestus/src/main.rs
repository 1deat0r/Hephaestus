//! `hephaestus` — the local CLI (MASTER_SPEC:42) carrying M1's exit line
//! (IMPLEMENTATION_PLAN:44): run and recover a deterministic fixture DAG.
//!
//! Usage:
//!   hephaestus fixture run         --state-dir <DIR>
//!   hephaestus fixture recover     --state-dir <DIR>
//!   hephaestus fixture mission-run --trace <FILE>
//!   hephaestus --help
//!
//! Exit codes: 0 success · 1 operational failure (bad state dir, refused
//! overwrite) · 2 usage error (unknown/missing arguments).
//!
//! Documentation lives here and in `--help`: README is MANIFEST-frozen
//! (81-file envelope) and its shell fences are gate-checked, so CLI docs
//! deliberately do not touch it (auto-workflow decision, cycle 9).

mod fixture;

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: hephaestus <fixture run|fixture recover> --state-dir <DIR>\n       hephaestus <fixture mission-run> --trace <FILE>\n       hephaestus --help\n";

fn print_help() {
    println!(
        "hephaestus — local CLI (M1 exit fixture)\n\n\
         \x20 Commands:\n\
         \x20   fixture run     --state-dir <DIR>   run the deterministic fixture DAG\n\
         \x20                                       (state dir must be empty or absent)\n\
         \x20   fixture recover --state-dir <DIR>   replay the recorded ledger and print\n\
         \x20                                       states plus the recovery plan\n\
         \x20   fixture mission-run --trace <FILE>   run the E2E mission chain over a\n\
         \x20                                       world-derived trace fixture and print\n\
         \x20                                       supported + negative receipts\n\n\
         \x20 Exit codes: 0 ok · 1 operational failure · 2 usage error\n\n{USAGE}"
    );
}

fn usage_error() -> ExitCode {
    let _ = std::io::stderr().write_all(USAGE.as_bytes());
    ExitCode::from(2)
}

fn trace_of(rest: &[String]) -> Result<PathBuf, ()> {
    match rest {
        [flag, file] if flag == "--trace" && !file.is_empty() && !file.starts_with('-') => {
            Ok(PathBuf::from(file))
        }
        _ => Err(()),
    }
}

fn state_dir_of(rest: &[String]) -> Result<PathBuf, ()> {
    match rest {
        [flag, dir] if flag == "--state-dir" && !dir.is_empty() && !dir.starts_with('-') => {
            Ok(PathBuf::from(dir))
        }
        _ => Err(()),
    }
}

/// Shared dispatch: parse args, run one command, print its JSON summary.
fn command(rest: &[String], body: fn(&std::path::Path) -> Result<String, String>) -> ExitCode {
    match state_dir_of(rest) {
        Err(()) => usage_error(),
        Ok(dir) => match body(&dir) {
            Err(e) => {
                let _ = writeln!(std::io::stderr(), "error: {e}");
                ExitCode::from(1)
            }
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
        },
    }
}

/// `fixture mission-run --trace <FILE>`: run the E2E chain twice over
/// the supplied world-derived trace (supported claim + honest negative)
/// and print both receipts as one JSON document.
fn mission_run(rest: &[String]) -> ExitCode {
    use hephaestus::dossier::record::NegativeResultKind;
    let path = match trace_of(rest) {
        Err(()) => return usage_error(),
        Ok(p) => p,
    };
    let text = match std::fs::read_to_string(&path) {
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: read {}: {e}", path.display());
            return ExitCode::from(1);
        }
        Ok(t) => t,
    };
    let supported = hephaestus::missionrun::run(&text, 0.25, None);
    let negative =
        hephaestus::missionrun::run(&text, 10.0, Some(NegativeResultKind::UnsupportedMechanism));
    match (supported, negative) {
        (Ok(s), Ok(n)) => {
            println!("{}", serde_json::json!({ "supported": s, "negative": n }));
            ExitCode::SUCCESS
        }
        (Err(e), _) | (_, Err(e)) => {
            let _ = writeln!(std::io::stderr(), "error: {e}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [help] if help == "--help" || help == "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        [fixture, rest @ ..] if fixture == "fixture" => match rest.first().map(String::as_str) {
            Some("run") => command(&rest[1..], |dir| {
                fixture::run(dir).map(|s| serde_json::to_string(&s).expect("summary serializes"))
            }),
            Some("recover") => command(&rest[1..], |dir| {
                fixture::recover(dir)
                    .map(|s| serde_json::to_string(&s).expect("summary serializes"))
            }),
            Some("mission-run") => mission_run(&rest[1..]),
            _ => usage_error(),
        },
        _ => usage_error(),
    }
}
