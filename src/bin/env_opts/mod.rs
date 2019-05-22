use std::ffi::*;
use std::path::*;

use structopt::clap::AppSettings;
use structopt::StructOpt;

use ndbam::*;

#[derive(StructOpt, Debug)]
#[structopt(raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct EnvOpts {
    /// Location of database
    #[structopt(
        short,
        long,
        name = "PATH",
        default_value = "/var/db/paludis/repositories/installed",
        parse(from_os_str = "parse_path_arg")
    )]
    pub location: PathBuf,

    /// Root of managed file-system
    #[structopt(
        short,
        long,
        default_value = "/",
        parse(from_os_str = "parse_root_arg")
    )]
    pub root: AnyRoot,
}

impl EnvOpts {
    pub fn ndbam(&self) -> NDBAM {
        NDBAM::new(&self.location)
    }
}

fn parse_path_arg(arg: &OsStr) -> PathBuf {
    Path::new(arg).canonicalize().expect("valid path")
}

fn parse_root_arg(arg: &OsStr) -> AnyRoot {
    RootAtBuf(parse_path_arg(arg))
}
