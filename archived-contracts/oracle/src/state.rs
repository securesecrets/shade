use shade_protocol::c_std::Storage;
use shade_protocol::storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use shade_protocol::contract_interfaces::{
    dex::dex,
    oracles::oracle::{IndexElement, OracleConfig},
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static DEX_PAIRS: &[u8] = b"dex_pairs";
pub static SSWAP_PAIRS: &[u8] = b"sswap_pairs";
pub static SIENNA_PAIRS: &[u8] = b"sienna_pairs";
pub static INDEX: &[u8] = b"index";

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<OracleConfig> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<OracleConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn dex_pairs_r(storage: &dyn Storage) -> ReadonlyBucket<Vec<dex::TradingPair>> {
    bucket_read(storage, DEX_PAIRS)
}

pub fn dex_pairs_w(storage: &mut dyn Storage) -> Bucket<Vec<dex::TradingPair>> {
    bucket(storage, DEX_PAIRS)
}

pub fn index_r(storage: &dyn Storage) -> ReadonlyBucket<Vec<IndexElement>> {
    bucket_read(storage, INDEX)
}

pub fn index_w(storage: &mut dyn Storage) -> Bucket<Vec<IndexElement>> {
    bucket(storage, INDEX)
}
