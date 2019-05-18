use std::path::PathBuf;

use structopt::clap::AppSettings;
use structopt::StructOpt;

use ndbam::NDBAM;

#[derive(StructOpt, Debug)]
#[structopt(raw(global_settings = "&[AppSettings::ColoredHelp]"))]
pub struct EnvOpts {
    /// Location of database
    #[structopt(
        short,
        long,
        name = "PATH",
        default_value = "/var/db/paludis/repositories/installed"
    )]
    pub location: PathBuf,

    /// Root of managed file-system
    #[structopt(short, long, default_value = "/")]
    pub root: PathBuf,
}

impl EnvOpts {
    pub fn canonicalize(&mut self) {
        self.root = {
            self.root
                .canonicalize()
                .expect("root should be a valid path")
        };

        self.location = {
            self.location
                .canonicalize()
                .expect("location should be a valid path")
        };
    }

    pub fn ndbam(&self) -> NDBAM {
        NDBAM::new(&self.location)
    }
}
