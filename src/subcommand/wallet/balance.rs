use {super::*, std::collections::BTreeSet};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub cardinal: u64,
  pub ordinal: u64,
  pub total: u64,
}

pub(crate) fn run(wallet: String, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  index.update()?;

  let client = bitcoin_rpc_client_for_wallet_command(wallet, &options)?;

  let unspent_outputs = get_unspent_outputs(&client, &index)?;

  let inscription_outputs = index
    .get_inscriptions(&unspent_outputs)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut cardinal = 0;
  let mut ordinal = 0;

  for (outpoint, amount) in unspent_outputs {
    let rune_balances = index.get_rune_balances_for_outpoint(outpoint)?;

    if inscription_outputs.contains(&outpoint) {
      ordinal += amount.to_sat();
    } else if !rune_balances.is_empty() {
    } else {
      cardinal += amount.to_sat();
    }
  }

  Ok(Box::new(Output {
    cardinal,
    ordinal,
    total: cardinal + ordinal,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn runes_and_runic_fields_are_not_present_if_none() {
    assert_eq!(
      serde_json::to_string(&Output {
        cardinal: 0,
        ordinal: 0,
        total: 0
      })
      .unwrap(),
      r#"{"cardinal":0,"ordinal":0,"total":0}"#
    );
  }
}
