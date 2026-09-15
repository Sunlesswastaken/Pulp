use std::process::Command;

fn pulp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pulp"))
}

#[test]
fn help_mentions_every_operation() {
    let output = pulp().arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for op in [
        "compress", "merge", "split", "remove", "extract", "password", "info",
    ] {
        assert!(text.contains(op), "--help should mention `{op}`");
    }
}

#[test]
fn bare_invocation_without_tty_prints_help() {
    // In the test harness stdin is a pipe, so `pulp` must not try to
    // open the TUI; it prints help and exits 0.
    let output = pulp().output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("PDF toolkit"));
}

#[test]
fn cli_operation_reports_not_implemented() {
    let output = pulp().args(["info", "whatever.pdf"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("not implemented"), "stderr: {err}");
}
