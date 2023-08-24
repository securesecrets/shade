use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use secret_toolkit::storage::{Item, Keymap};

use libraries::types::Bytes32;

pub static CONFIG: Item<State> = Item::new(b"config");
pub static BIN_MAP: Keymap<u32, Bytes32> = Keymap::new(b"bins");

#[cw_serde]
pub struct State {
    // TODO: use canonical addresses
    pub creator: Addr,
}
