use cosmwasm_std::{Event, StdError, StdResult};

struct AttributeKey;

impl AttributeKey {
    pub const AMM_PAIR_ADDRESS: &'static str = "amm_pair_address";
    pub const LP_TOKEN_ADDRESS: &'static str = "lp_token_address";
    pub const STAKING_CONTRACT_ADDRESS: &'static str = "staking_contract_address";
}

pub struct EventsParser;

impl EventsParser {
    /// Searches through custom events for a specific attribute key and returns its value if it exists. Custom events have type 'wasm'.
    pub fn may_find_custom_value(events: &[Event], attribute_key: &str) -> Option<String> {
        for event in events {
            if event.ty == "wasm" {
                for attribute in &event.attributes {
                    if attribute.key == attribute_key {
                        return Some(attribute.value.clone());
                    }
                }
            }
        }
        None
    }
    pub fn try_find_custom_value(events: &[Event], attribute_key: &str) -> StdResult<String> {
        if let Some(value) = Self::may_find_custom_value(events, attribute_key) {
            Ok(value)
        } else {
            Err(StdError::generic_err(format!(
                "Could not find custom attribute with {attribute_key}"
            )))
        }
    }
}
