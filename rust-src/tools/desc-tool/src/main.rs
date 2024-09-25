
use clap::App;

mod h;
mod create;
mod modify;
mod show;

use create::{create_desc, create_subcommand};
use modify::{modify_desc, modify_subcommand};
use show::{show_desc, show_subcommand};

fn main() {
    let matches = 
        App::new("desc-tool").version("1.0").about("tool to create or show desc files")
            .subcommand(create_subcommand())
            .subcommand(modify_subcommand())
            .subcommand(show_subcommand())
            .get_matches();

    if let Some(command) = matches.subcommand() {
        match command {
            ("create", matches) => {
                create_desc(matches)
            },
            ("modify", matches) => {
                modify_desc(matches);
            },
            ("show", matches) => {
                show_desc(matches)
            }
            _v @ _ => {
            }
        }
    } else {
        print!(
            r#"
USAGE: 
    desc-tool [SUBCOMMAND]
For more information try --help or -h"#);
    }

}
