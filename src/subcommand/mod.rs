use super::*;

pub mod decode;
pub mod epochs;
pub mod index;
pub mod subsidy;
pub mod supply;
pub mod wallet;

use crate::index::get_tx_limits;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "Decode a transaction")]
  Decode(decode::Decode),
  #[command(about = "List the first satoshis of each reward epoch")]
  Epochs,
  #[command(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[command(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[command(about = "Display Bitcoin supply information")]
  Supply,
  #[command(about = "Wallet commands")]
  Wallet(wallet::Wallet),
  #[command(about = "List max transfer counts")]
  MaxTransferCounts,
}

fn max_transfer_counts() -> SubcommandResult {
  // create a dictionary. set 'default' to 2 and 'brc20-approve-conditional' to 5
  let max_transfer_counts = get_tx_limits();
  Ok(Box::new(max_transfer_counts))
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self {
      Self::Decode(decode) => decode.run(options),
      Self::Epochs => epochs::run(),
      Self::Index(index) => index.run(options),
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Supply => supply::run(),
      Self::Wallet(wallet) => wallet.run(options),
      Self::MaxTransferCounts => max_transfer_counts(),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Empty {}

pub(crate) trait Output: Send {
  fn print_json(&self);
}

impl<T> Output for T
where
  T: Serialize + Send,
{
  fn print_json(&self) {
    serde_json::to_writer_pretty(io::stdout(), self).ok();
    println!();
  }
}

pub(crate) type SubcommandResult = Result<Box<dyn Output>>;
