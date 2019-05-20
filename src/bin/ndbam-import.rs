mod env_opts;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use structopt::clap::AppSettings;
use structopt::StructOpt;

use env_opts::*;

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
    fn image(&self) -> Cow<Path> {
        if let Some(ref image) = self.image {
            Cow::from(image)
        } else {
            Cow::from(std::env::current_dir().unwrap())
        }
    }
}

fn main() {
    let opts = {
        let mut raw_opts = Opts::from_args();
        raw_opts.env.canonicalize();
        raw_opts
    };

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
        .merge(opts.image().as_ref(), &opts.env.root).unwrap();
}
