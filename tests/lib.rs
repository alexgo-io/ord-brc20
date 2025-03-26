#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  bitcoin::{
    address::{Address, NetworkUnchecked},
    blockdata::constants::COIN_VALUE,
    Network, OutPoint, Txid,
  },
  executable_path::executable_path,
  ord::{InscriptionId, Rune, SatPoint},
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  std::{
    collections::BTreeMap,
    fs,
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    str::{self, FromStr},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
  test_bitcoincore_rpc::Sent,
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

const RUNE: u128 = 99246114928149462;

type Inscribe = ord::subcommand::wallet::inscribe::Output;
type Etch = ord::subcommand::wallet::etch::Output;

fn create_wallet(rpc_server: &test_bitcoincore_rpc::Handle) {
  CommandBuilder::new(format!("--chain {} wallet create", rpc_server.network()))
    .rpc_server(rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::create::Output>();
}

fn etch(rpc_server: &test_bitcoincore_rpc::Handle, rune: Rune) -> Etch {
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(
    format!(
    "--index-runes --regtest wallet etch --rune {} --divisibility 0 --fee-rate 0 --supply 1000 --symbol ¢",
    rune
    )
  )
  .rpc_server(rpc_server)
  .run_and_deserialize_output();

  rpc_server.mine_blocks(1);

  output
}

fn inscribe(rpc_server: &test_bitcoincore_rpc::Handle) -> (InscriptionId, Txid) {
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --file foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(output.inscriptions.len(), 1);

  (output.inscriptions[0].id, output.reveal)
}

mod command_builder;
mod expected;
mod test_server;

mod core;
mod decode;
mod epochs;
mod index;
mod subsidy;
mod supply;
mod version;
mod wallet;
