extern crate ansi_term;
extern crate clap;
extern crate shellexpand;
extern crate toml;
extern crate winapi;
extern crate advapi32;
extern crate kernel32;

mod cli;
mod dotfiles;
mod entry;
mod util;
#[cfg(windows)]
mod privilege;

use std::env;
use std::path::Path;
use dotfiles::Dotfiles;


pub fn main() {
  if env::var("HOME").is_err() {
    env::set_var("HOME", env::home_dir().unwrap().to_str().unwrap());
  }

  let mut app = App::new();
  std::process::exit(app.main());
}


struct App {
  dotfiles: Dotfiles,
}

impl App {
  pub fn new() -> App {
    let dotdir = env::var("DOT_DIR").expect("$DOT_DIR is not set.");
    let dotdir = Path::new(&dotdir).to_path_buf();
    env::set_var("dotdir", dotdir.as_os_str());

    let dotfiles = Dotfiles::new(dotdir);

    App { dotfiles: dotfiles }
  }

  pub fn main(&mut self) -> i32 {
    let matches = cli::build_cli().get_matches();
    match matches.subcommand() {
      ("check", Some(args)) => {
        let verbose = args.is_present("verbose");
        self.command_check(verbose)
      }
      ("link", Some(args)) => {
        let dry_run = args.is_present("dry-run");
        let verbose = args.is_present("verbose");
        self.command_link(dry_run, verbose)
      }
      ("clean", Some(args)) => {
        let dry_run = args.is_present("dry-run");
        let verbose = args.is_present("verbose");
        self.command_clean(dry_run, verbose)
      }
      ("root", _) => self.command_root(),
      (_, _) => unreachable!(),
    }
  }

  fn command_root(&self) -> i32 {
    println!("{}", self.dotfiles.root_dir().display());
    0
  }

  fn command_check(&self, verbose: bool) -> i32 {
    let mut num_unhealth = 0;
    for entry in self.dotfiles.entries() {
      if entry.check(verbose).unwrap() == false {
        num_unhealth += 1;
      }
    }
    num_unhealth
  }

  fn command_link(&self, dry_run: bool, verbose: bool) -> i32 {
    if !util::enable_symlink_privilege() {
      panic!("failed to enable SeCreateSymbolicLinkPrivilege");
    }

    for entry in self.dotfiles.entries() {
      entry.mklink(dry_run, verbose).unwrap();
    }
    0
  }

  fn command_clean(&self, dry_run: bool, verbose: bool) -> i32 {
    for entry in self.dotfiles.entries() {
      entry.unlink(dry_run, verbose).unwrap();
    }
    0
  }
}
