#[derive(Copy, Clone)]
pub(crate) enum Charm {
  Coin,
  Cursed,
  Epic,
  Legendary,
  Lost,
  Nineball,
  Rare,
  Reinscription,
  Unbound,
  Uncommon,
}

impl Charm {
  fn flag(self) -> u16 {
    1 << self as u16
  }

  pub(crate) fn set(self, charms: &mut u16) {
    *charms |= self.flag();
  }
}
