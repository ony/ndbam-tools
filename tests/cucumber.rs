#[macro_use]
extern crate cucumber_rust;

use std::default::Default;
use std::process::Command;
use std::path::{Path, PathBuf};

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

    fn real_path(&self, path: &Path) -> PathBuf {
        let mut components = path.components();
        components.next();
        self.root.path().join(components.as_path())
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
        then "failure" |world, _step| { world.cmd_assert().failure(); };

        then "no output" |world, _step| { world.cmd_assert().stdout(""); };

        then "output is:" |world, step| {
            let expected = step.docstring().unwrap().to_string();
            world.cmd_assert().stdout(expected + "\n");
        };
    });
}

mod content_steps {
    use super::*;
    use std::fs;

    steps!(super::Env => {
        given regex r"^file (.+)$" (PathBuf) |world, path, step| {
            let real_path = world.real_path(&path);
            create_dir_for(&real_path);
            if let Some(content) = step.docstring() {
                assert!(!content.contains('<') && !content.contains('>'),
                    "variables are not yet supported. cucumber test skipped"); // magic trail to skip
                fs::write(&real_path, content)
            } else {
                fs::write(&real_path, "dummy")
            }.expect(format!("write to {:?} (original {:?})", &real_path, &path).as_str());
        };

        given regex r"^directory (.+)$" (PathBuf) |world, path, _step| {
            fs::create_dir_all(world.real_path(&path)).unwrap();
        };

        given regex r"^symlink (.+) to (.+)$" (PathBuf, PathBuf) |world, path, target, _step| {
            let real_path = world.real_path(&path);
            create_dir_for(&real_path);
            fs::soft_link(&target, &real_path)
                .expect(format!("symlink at {:?} {:?}", &real_path, &path).as_str());
        };
    });

    fn create_dir_for(path: &Path) {
        path.parent().map(|parent| fs::create_dir_all(parent).unwrap() );
    }
}

cucumber! {
    features: "./features",
    world: Env,
    steps: &[basic_steps::steps, content_steps::steps]
}
