use crate::cli::SourcePathArgs;
use crate::error::{AppError, AppResult};
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;

pub fn path(args: &SourcePathArgs) -> AppResult<()> {
    let output = opensrc_command(args).output().map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            AppError::new(
                "E_SOURCE_UNAVAILABLE",
                "opensrc is not installed or not on PATH; install it with `npm install -g opensrc`",
            )
        } else {
            AppError::with_detail("E_SOURCE_FETCH", "failed to run opensrc", err.to_string())
        }
    })?;

    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr).map_err(|err| {
            AppError::with_detail(
                "E_OUTPUT",
                "failed to write opensrc stderr",
                err.to_string(),
            )
        })?;
    }

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if detail.is_empty() {
            format!("opensrc exited with status {}", output.status)
        } else {
            format!("opensrc failed: {detail}")
        };
        return Err(AppError::new("E_SOURCE_FETCH", message));
    }

    io::stdout().write_all(&output.stdout).map_err(|err| {
        AppError::with_detail(
            "E_OUTPUT",
            "failed to write opensrc stdout",
            err.to_string(),
        )
    })
}

fn opensrc_command(args: &SourcePathArgs) -> Command {
    let binary =
        std::env::var_os("GITQUARRY_OPENSRC_BIN").unwrap_or_else(|| OsString::from("opensrc"));
    let mut command = Command::new(binary);
    command.arg("path");

    if let Some(cwd) = &args.cwd {
        command.arg("--cwd").arg(cwd);
    }

    if args.verbose {
        command.arg("--verbose");
    }

    command.arg(&args.spec);
    command
}
