use std::path::Path;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;

use assert_fs::fixture::TempDir;

use std::process::Command;

fn sample(name: &str) -> TempDir {
    let temp = assert_fs::TempDir::new().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples").join(name);
    temp.copy_from(&source, &["*"]).expect(&format!("Fail to copy from {:?}", &source));
    temp
}

fn cmd_check(root: &TempDir) -> std::process::Command {
    let mut cmd = Command::cargo_bin("ndbam-check").unwrap();
    cmd.env("RUST_BACKTRACE", "1")
        .arg("--root").arg(root.path())
        .arg("--location").arg(root.path().join("var").join("db").join("ndbam"));
    cmd
}

#[test]
fn test_basic() {
    let sample = sample("basic");

    cmd_check(&sample).arg("bye").assert().success().stdout("bye - Not found\n");

    cmd_check(&sample).arg("empty").assert().success().stdout("");

    cmd_check(&sample).arg("--show-size").arg("empty")
        .assert().success().stdout(
            "empty-0:0\
            \n  # Size: 0 B\n");

    cmd_check(&sample).arg("--allow-mtime").arg("hello").assert().success().stdout("");

    cmd_check(&sample).arg("--allow-mtime").arg("--show-size").arg("amended")
        .assert().success().stdout(
            "amended-0:0\
            \n  C /amended.txt Content changed\
            \n  # Size: 0 B\n");
}
