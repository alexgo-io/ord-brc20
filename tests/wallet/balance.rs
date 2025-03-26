use {super::*, ord::subcommand::wallet::balance::Output};

#[test]
fn wallet_balance() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>()
      .cardinal,
    0
  );

  rpc_server.mine_blocks(1);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 50 * COIN_VALUE,
      ordinal: 0,
      total: 50 * COIN_VALUE,
    }
  );
}

#[test]
fn inscribed_utxos_are_deducted_from_cardinal() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 0,
      ordinal: 0,
      total: 0,
    }
  );

  inscribe(&rpc_server);

  assert_eq!(
    CommandBuilder::new("wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<Output>(),
    Output {
      cardinal: 100 * COIN_VALUE - 10_000,
      ordinal: 10_000,
      total: 100 * COIN_VALUE,
    }
  );
}
