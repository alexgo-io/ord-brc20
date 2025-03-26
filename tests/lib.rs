#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected},
  bitcoin::OutPoint,
  executable_path::executable_path,
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  serde::de::DeserializeOwned,
  std::{
    fs,
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::{self},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
};

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}

mod command_builder;
mod expected;

mod core;
mod decode;
mod epochs;
mod index;
mod subsidy;
mod supply;
mod version;
