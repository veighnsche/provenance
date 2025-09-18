use std::{env, process::{Command, ExitCode}};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.get(0).map(String::as_str).unwrap_or("check-all");
    match cmd {
        "check-all" => seq(&[
            &["cargo","fmt","--all","--","--check"],
            // Fail only on Clippy warnings; allow rustc warnings for now
            &["cargo","clippy","--all-targets","--all-features","--","-D","clippy::all"],
            &["cargo","check","--all-features"],
            &["cargo","clippy","--manifest-path","crates/proofdown_parser/Cargo.toml",
              "--all-targets","--all-features","--","-D","clippy::all"],
            &["cargo","check","--manifest-path","crates/proofdown_parser/Cargo.toml","--all-features"],
        ]),
        "test-all" => seq(&[
            &["cargo","test","--all-features","--","--nocapture"],
            &["cargo","test","--manifest-path","crates/proofdown_parser/Cargo.toml","--all-features","--","--nocapture"],
        ]),
        "ci-integration-build" => seq(&[
            &["cargo","build","-p","provenance_ssg","--features","external_pml"],
        ]),
        "e2e" => seq(&[
            // Install e2e dependencies (force reinstallation non-interactively) and run Cypress headless
            &["pnpm","-C","e2e","install","--force"],
            &["pnpm","-C","e2e","run","e2e"],
        ]),
        "e2e-open" => seq(&[
            // Install e2e dependencies (force reinstallation non-interactively) and open Cypress UI
            &["pnpm","-C","e2e","install","--force"],
            &["pnpm","-C","e2e","run","e2e:open"],
        ]),
        other => {
            eprintln!("unknown subcommand: {}", other);
            ExitCode::from(2)
        }
    }
}

fn seq(cmds: &[&[&str]]) -> ExitCode {
    for c in cmds {
        eprintln!("+ {}", c.join(" "));
        let status = Command::new(c[0]).args(&c[1..]).status().expect("spawn failed");
        if !status.success() {
            return ExitCode::from(status.code().unwrap_or(1) as u8);
        }
    }
    ExitCode::SUCCESS
}
