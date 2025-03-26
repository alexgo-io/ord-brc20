pub(crate) use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script, script::PushBytesBuf},
    Witness,
  },
};

macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

pub(crate) fn txid(n: u64) -> Txid {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  hex.repeat(64).parse().unwrap()
}

#[derive(Default, Debug)]
pub(crate) struct InscriptionTemplate {
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) pointer: Option<u64>,
}

impl InscriptionId {
  #[cfg(test)]
  pub(crate) fn value(self) -> Vec<u8> {
    let index = self.index.to_le_bytes();
    let mut index_slice = index.as_slice();

    while index_slice.last().copied() == Some(0) {
      index_slice = &index_slice[0..index_slice.len() - 1];
    }

    self
      .txid
      .to_byte_array()
      .iter()
      .chain(index_slice)
      .copied()
      .collect()
  }
}

impl From<InscriptionTemplate> for Inscription {
  fn from(template: InscriptionTemplate) -> Self {
    Self {
      parent: template.parent.map(|id| id.value()),
      pointer: template.pointer.map(Inscription::pointer_value),
      ..Default::default()
    }
  }
}

pub(crate) fn inscription(content_type: &str, body: impl AsRef<[u8]>) -> Inscription {
  Inscription::new(Some(content_type.into()), Some(body.as_ref().into()))
}

pub(crate) fn inscription_id(n: u32) -> InscriptionId {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  format!("{}i{n}", hex.repeat(64)).parse().unwrap()
}

pub(crate) fn envelope(payload: &[&[u8]]) -> Witness {
  let mut builder = script::Builder::new()
    .push_opcode(opcodes::OP_FALSE)
    .push_opcode(opcodes::all::OP_IF);

  for data in payload {
    let mut buf = PushBytesBuf::new();
    buf.extend_from_slice(data).unwrap();
    builder = builder.push_slice(buf);
  }

  let script = builder.push_opcode(opcodes::all::OP_ENDIF).into_script();

  Witness::from_slice(&[script.into_bytes(), Vec::new()])
}
