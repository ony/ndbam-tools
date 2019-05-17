#[macro_use]
extern crate cucumber_rust;

use std::default::Default;
use std::process::Command;

use assert_cmd::prelude::*;

use assert_fs::fixture::TempDir;

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
            let args = shellwords::split(&trail).unwrap();
            let mut cmd = Command::cargo_bin(program).unwrap();
            cmd.env("RUST_BACKTRACE", "1")
                .arg("--root").arg(world.root.path())
                .arg("--location").arg(world.root.path().join("var").join("db").join("ndbam"))
                .args(args);
            world.cmd_output = Some(cmd.output().unwrap());
        };

        then "success" |world, _step| { world.cmd_assert().success(); };

        then "no output" |world, _step| { world.cmd_assert().stdout(""); };

        then "output is:" |world, step| {
            let expected = step.docstring().unwrap().to_string();
            world.cmd_assert().stdout(expected + "\n");
        };
    });
}

cucumber! {
    features: "./features",
    world: Env,
    steps: &[basic_steps::steps]
}
