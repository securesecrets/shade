use cosmwasm_std::{StdError, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FlexibleMsg {
    pub msg: String,
    pub arguments: u16,
}

impl FlexibleMsg {
    pub fn new(msg: String, msg_variable: &str) -> FlexibleMsg {
        FlexibleMsg {
            msg: msg.clone(),
            arguments: msg.matches(msg_variable).count() as u16,
        }
    }

    pub fn create_msg(&self, args: Vec<String>, msg_variable: &str) -> StdResult<String> {
        if args.len() as u16 != self.arguments {
            return Err(StdError::generic_err(format!(
                "Msg expected {:?} arguments; received {:?}",
                self.arguments,
                args.len()
            )));
        }

        let mut msg = self.msg.clone();
        for arg in args.iter() {
            msg = msg.replacen(msg_variable, arg, 1);
        }
        Ok(msg)
    }
}
