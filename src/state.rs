use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const PAYEES: Item<Vec<Addr>> = Item::new("payees");
pub const TOTAL_SHARES: Item<u64> = Item::new("total_shares");
pub const TOTAL_RELEASED: Item<Uint128> = Item::new("total_released");
pub const SHARES: Map<&Addr, u64> = Map::new("shares");
pub const RELEASED: Map<&Addr, Uint128> = Map::new("released");
