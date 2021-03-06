use cucumber_rust::{cucumber, steps};

use std::default::Default;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::fixture::{TempDir, ChildPath, PathChild};
use predicates::prelude::*;

pub struct Env {
    root: TempDir,
    cmd_output: Option<std::process::Output>,
}

impl Env {
    fn cmd_assert(&self) -> assert_cmd::assert::Assert {
        self.cmd_output
            .clone()
            .expect("missing command execution?")
            .assert()
    }

    fn child_path(&self, path: &Path) -> ChildPath {
        let mut components = path.components();
        components.next();
        self.root.child(components.as_path())
    }
}

impl cucumber_rust::World for Env {}
impl Default for Env {
    fn default() -> Env {
        // This function is called every time a new scenario is started
        Env {
            root: assert_fs::TempDir::new().unwrap(),
            cmd_output: None,
        }
    }
}

mod basic_steps {
    use super::*;
    use assert_fs::prelude::*;

    steps!(super::Env => {
        given regex r"^sample with (.+) content$" (String) |world, name, _step| {
            let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples").join(name);
            world.root.copy_from(&source, &["*"]).expect(&format!("Fail to copy from {:?}", &source));
        };

        when regex r"^run (\S+)(.*)$" (String, String) |world, program, trail, _step| {
            let location = world.root.path().join("var").join("db").join("ndbam");
            let args: Vec<String> = {
                shellwords::split(&trail).unwrap().iter()
                    .map(|arg| {
                        // TODO: proper expand
                        arg.replace("${root}", world.root.path().to_str().unwrap())
                            .replace("${location}", location.to_str().unwrap())
                    })
                    .collect()
            };

            let mut cmd = if program.starts_with("ndbam-") {
                Command::cargo_bin(&program).unwrap()
            } else {
                Command::new(&program)
            };

            if program.starts_with("ndbam-") {
                cmd.env("RUST_BACKTRACE", "1")
                    .arg("--root").arg(world.root.path())
                    .arg("--location").arg(location)
                    .args(args);
            } else {
                cmd.args(args);
            }

            world.cmd_output = Some(cmd.output().unwrap());
        };

        then "success" |world, _step| { world.cmd_assert().success(); };
        then "failure" |world, _step| { world.cmd_assert().failure(); };

        then "no output" |world, _step| { world.cmd_assert().stdout(""); };

        then "output is:" |world, step| {
            let expected = step.docstring().unwrap().to_string();
            world.cmd_assert().stdout(expected + "\n");
        };

        then regex r"output contains:\s*(.*)" (String) |world, needle, _step| {
            world.cmd_assert().stdout(predicate::str::contains(needle));
        };

        then regex r"errors do(?:es)? not contains?:\s*(.*)" (String) |world, needle, _step| {
            world.cmd_assert().stderr(predicate::str::contains(needle).not());
        };
    });
}

mod content_steps;

cucumber! {
    features: "./features",
    world: Env,
    steps: &[basic_steps::steps, content_steps::steps]
}
