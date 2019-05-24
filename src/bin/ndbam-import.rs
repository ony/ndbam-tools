mod env_opts;

use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use env_opts::*;
use ndbam::*;

const DEFAULT_REPO_PATH : &'static str = "/var/db/paludis/repositories/unpackaged";

#[derive(StructOpt, Debug)]
#[structopt(raw(global_settings = "&[AppSettings::ColoredHelp]"))]
struct Opts {
    #[structopt(flatten)]
    env: EnvOpts,

    /// Path to directory containing the image to install (current by default)
    #[structopt(long, short)]
    image: Option<PathBuf>,

    /// Do not perform actual modifications
    #[structopt(long = "dry-run", short = "n")]
    dry_run: bool,

    /// Name of the package (with category if applicable)
    package_name: String,

    #[structopt(default_value = "0")]
    version: String,

    #[structopt(default_value = "0")]
    slot: String,
}

impl Opts {
    fn image(&self) -> impl RootPath {
        if let Some(ref image) = self.image {
            RootAtBuf(image.to_owned())
        } else {
            RootAtBuf(std::env::current_dir().unwrap())
        }
    }
}

fn main() {
    let opts =  Opts::from_args();

    let reg = opts.env.ndbam();
    assert!(
        !reg.versions_of(&opts.package_name).is_some(),
        "Upgrades and slots are not supported yet"
    );

    if opts.dry_run {
        println!("Dry-run. No actions.");
        return;
    }

    reg.new_package_version(&opts.package_name, &opts.version, &opts.slot)
        .merge(&opts.image(), &opts.env.root).unwrap();
}
