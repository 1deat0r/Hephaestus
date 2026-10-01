//! `hephaestus` — the local CLI (MASTER_SPEC:42) carrying M1's exit line
//! (IMPLEMENTATION_PLAN:44): run and recover a deterministic fixture DAG.
//!
//! Usage:
//!   hephaestus fixture run    --state-dir <DIR>
//!   hephaestus fixture recover --state-dir <DIR>
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

const USAGE: &str =
    "usage: hephaestus <fixture run|fixture recover> --state-dir <DIR>\n       hephaestus --help\n";

fn print_help() {
    println!(
        "hephaestus — local CLI (M1 exit fixture)\n\n\
         \x20 Commands:\n\
         \x20   fixture run     --state-dir <DIR>   run the deterministic fixture DAG\n\
         \x20                                       (state dir must be empty or absent)\n\
         \x20   fixture recover --state-dir <DIR>   replay the recorded ledger and print\n\
         \x20                                       states plus the recovery plan\n\n\
         \x20 Exit codes: 0 ok · 1 operational failure · 2 usage error\n\n{USAGE}"
    );
}

fn usage_error() -> ExitCode {
    let _ = std::io::stderr().write_all(USAGE.as_bytes());
    ExitCode::from(2)
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
            _ => usage_error(),
        },
        _ => usage_error(),
    }
}
