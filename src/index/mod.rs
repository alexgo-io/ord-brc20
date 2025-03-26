use {
  self::{
    entry::{
      Entry, HeaderValue, InscriptionEntry, InscriptionEntryValue, InscriptionIdValue,
      OutPointValue, SatPointValue,
    },
    reorg::*,
    updater::Updater,
  },
  super::*,
  bitcoin::block::Header,
  bitcoincore_rpc::Client,
  indicatif::{ProgressBar, ProgressStyle},
  log::log_enabled,
  ord_bitcoincore_rpc as bitcoincore_rpc,
  redb::{
    Database, DatabaseError, MultimapTable, MultimapTableDefinition, ReadableMultimapTable,
    ReadableTable, RepairSession, StorageError, Table, TableDefinition, WriteTransaction,
  },
  std::{
    collections::HashMap,
    io::Write,
    sync::{Mutex, Once},
  },
  sysinfo::System,
};

pub(crate) mod entry;
mod fetcher;
mod reorg;
mod rtx;
mod updater;

pub use updater::get_tx_limits;

const SCHEMA_VERSION: u64 = 99100016;

macro_rules! define_table {
  ($name:ident, $key:ty, $value:ty) => {
    const $name: TableDefinition<$key, $value> = TableDefinition::new(stringify!($name));
  };
}

macro_rules! define_multimap_table {
  ($name:ident, $key:ty, $value:ty) => {
    const $name: MultimapTableDefinition<$key, $value> =
      MultimapTableDefinition::new(stringify!($name));
  };
}

define_multimap_table! { SATPOINT_TO_SEQUENCE_NUMBER, &SatPointValue, u32 }
define_multimap_table! { SAT_TO_SEQUENCE_NUMBER, u64, u32 }
define_table! { HEIGHT_TO_BLOCK_HEADER, u32, &HeaderValue }
define_table! { HEIGHT_TO_LAST_SEQUENCE_NUMBER, u32, u32 }
define_table! { INSCRIPTION_ID_TO_SEQUENCE_NUMBER, InscriptionIdValue, u32 }
define_table! { INSCRIPTION_ID_TO_TXCNT, InscriptionIdValue, i64 }
define_table! { OUTPOINT_TO_VALUE, &OutPointValue, u64}
define_table! { SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, u32, InscriptionEntryValue }
define_table! { STATISTIC_TO_COUNT, u64, u64 }
define_table! { WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP, u32, u128 }

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub(crate) enum Statistic {
  Schema = 0,
  BlessedInscriptions,
  Commits,
  CursedInscriptions,
  IndexRunes,
  IndexSats,
  LostSats,
  OutputsTraversed,
  ReservedRunes,
  Runes,
  SatRanges,
  UnboundInscriptions,
  IndexTransactions,
}

impl Statistic {
  fn key(self) -> u64 {
    self.into()
  }
}

impl From<Statistic> for u64 {
  fn from(statistic: Statistic) -> Self {
    statistic as u64
  }
}

trait BitcoinCoreRpcResultExt<T> {
  fn into_option(self) -> Result<Option<T>>;
}

impl<T> BitcoinCoreRpcResultExt<T> for Result<T, bitcoincore_rpc::Error> {
  fn into_option(self) -> Result<Option<T>> {
    match self {
      Ok(ok) => Ok(Some(ok)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { message, .. },
      )))
        if message.ends_with("not found") =>
      {
        Ok(None)
      }
      Err(err) => Err(err.into()),
    }
  }
}

pub struct Index {
  client: Client,
  database: Database,
  durability: redb::Durability,
  first_inscription_height: u32,
  height_limit: Option<u32>,
  options: Options,
  unrecoverably_reorged: AtomicBool,
}

impl Index {
  pub fn open(options: &Options) -> Result<Self> {
    let client = options.bitcoin_rpc_client(None)?;

    let path = options
      .index
      .clone()
      .unwrap_or(options.data_dir().clone().join("index.redb"));

    if let Err(err) = fs::create_dir_all(path.parent().unwrap()) {
      bail!(
        "failed to create data dir `{}`: {err}",
        path.parent().unwrap().display()
      );
    }

    let db_cache_size = match options.db_cache_size {
      Some(db_cache_size) => db_cache_size,
      None => {
        let mut sys = System::new();
        sys.refresh_memory();
        usize::try_from(sys.total_memory() / 4)?
      }
    };

    log::info!("Setting DB cache size to {} bytes", db_cache_size);

    let durability = if cfg!(test) {
      redb::Durability::None
    } else {
      redb::Durability::Immediate
    };

    let index_path = path.clone();
    let once = Once::new();
    let progress_bar = Mutex::new(None);

    let database = match Database::builder()
      .set_cache_size(db_cache_size)
      .set_repair_callback(move |progress: &mut RepairSession| {
        once.call_once(|| println!("Index file `{}` needs recovery. This can take a long time, especially for the --index-sats index.", index_path.display()));

        if !(cfg!(test) || log_enabled!(log::Level::Info) || integration_test()) {
          let mut guard = progress_bar.lock().unwrap();

          let progress_bar = guard.get_or_insert_with(|| {
            let progress_bar = ProgressBar::new(100);
            progress_bar.set_style(
              ProgressStyle::with_template("[repairing database] {wide_bar} {pos}/{len}").unwrap(),
            );
            progress_bar
          });

          #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
          progress_bar.set_position((progress.progress() * 100.0) as u64);
        }
      })
      .open(&path)
    {
      Ok(database) => {
        {
          let tx = database.begin_read()?;
          let statistics = tx.open_table(STATISTIC_TO_COUNT)?;

          let schema_version = statistics
            .get(&Statistic::Schema.key())?
            .map(|x| x.value())
            .unwrap_or(0);

          match schema_version.cmp(&SCHEMA_VERSION) {
            cmp::Ordering::Less =>
              bail!(
                "index at `{}` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
                path.display()
              ),
            cmp::Ordering::Greater =>
              bail!(
                "index at `{}` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
                path.display()
              ),
            cmp::Ordering::Equal => {
            }
          }
        }

        database
      }
      Err(DatabaseError::Storage(StorageError::Io(error)))
        if error.kind() == io::ErrorKind::NotFound =>
      {
        let database = Database::builder()
          .set_cache_size(db_cache_size)
          .create(&path)?;

        let mut tx = database.begin_write()?;

        tx.set_durability(durability);

        tx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
        tx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?;
        tx.open_table(HEIGHT_TO_BLOCK_HEADER)?;
        tx.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?;
        tx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
        tx.open_table(INSCRIPTION_ID_TO_TXCNT)?;
        tx.open_table(OUTPOINT_TO_VALUE)?;
        tx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
        tx.open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?;

        {
          let mut statistics = tx.open_table(STATISTIC_TO_COUNT)?;
          Self::set_statistic(&mut statistics, Statistic::Schema, SCHEMA_VERSION)?;
        }

        tx.commit()?;

        database
      }
      Err(error) => bail!("failed to open index: {error}"),
    };

    Ok(Self {
      client,
      database,
      durability,
      first_inscription_height: options.first_inscription_height(),
      height_limit: options.height_limit,
      options: options.clone(),
      unrecoverably_reorged: AtomicBool::new(false),
    })
  }

  pub(crate) fn update(&self) -> Result {
    let mut updater = Updater::new(self)?;

    loop {
      match updater.update_index() {
        Ok(ok) => return Ok(ok),
        Err(err) => {
          log::info!("{}", err.to_string());

          match err.downcast_ref() {
            Some(&ReorgError::Recoverable { height, depth }) => {
              Reorg::handle_reorg(self, height, depth)?;

              updater = Updater::new(self)?;
            }
            Some(&ReorgError::Unrecoverable) => {
              self
                .unrecoverably_reorged
                .store(true, atomic::Ordering::Relaxed);
              return Err(anyhow!(ReorgError::Unrecoverable));
            }
            _ => return Err(err),
          };
        }
      }
    }
  }

  fn begin_read(&self) -> Result<rtx::Rtx> {
    Ok(rtx::Rtx(self.database.begin_read()?))
  }

  fn begin_write(&self) -> Result<WriteTransaction> {
    let mut tx = self.database.begin_write()?;
    tx.set_durability(self.durability);
    Ok(tx)
  }

  fn increment_statistic(wtx: &WriteTransaction, statistic: Statistic, n: u64) -> Result {
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    let value = statistic_to_count
      .get(&(statistic.key()))?
      .map(|x| x.value())
      .unwrap_or_default()
      + n;
    statistic_to_count.insert(&statistic.key(), &value)?;
    Ok(())
  }

  pub(crate) fn set_statistic(
    statistics: &mut Table<u64, u64>,
    statistic: Statistic,
    value: u64,
  ) -> Result<()> {
    statistics.insert(&statistic.key(), &value)?;
    Ok(())
  }

  pub(crate) fn block_count(&self) -> Result<u32> {
    self.begin_read()?.block_count()
  }

  pub(crate) fn block_hash(&self, height: Option<u32>) -> Result<Option<BlockHash>> {
    self.begin_read()?.block_hash(height)
  }

  fn inscriptions_on_output<'a: 'tx, 'tx>(
    satpoint_to_sequence_number: &'a impl ReadableMultimapTable<&'static SatPointValue, u32>,
    sequence_number_to_inscription_entry: &'a impl ReadableTable<u32, InscriptionEntryValue>,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let start = SatPoint {
      outpoint,
      offset: 0,
    }
    .store();

    let end = SatPoint {
      outpoint,
      offset: u64::MAX,
    }
    .store();

    let mut inscriptions = Vec::new();

    for range in satpoint_to_sequence_number.range::<&[u8; 44]>(&start..=&end)? {
      let (satpoint, sequence_numbers) = range?;
      for sequence_number_result in sequence_numbers {
        let sequence_number = sequence_number_result?.value();
        let entry = sequence_number_to_inscription_entry
          .get(sequence_number)?
          .unwrap();
        inscriptions.push((
          sequence_number,
          SatPoint::load(*satpoint.value()),
          InscriptionEntry::load(entry.value()).id,
        ));
      }
    }

    inscriptions.sort_by_key(|(sequence_number, _, _)| *sequence_number);

    Ok(
      inscriptions
        .into_iter()
        .map(|(_sequence_number, satpoint, inscription_id)| (satpoint, inscription_id))
        .collect(),
    )
  }
}
