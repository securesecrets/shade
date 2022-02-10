use cosmwasm_std::StdError;
use schemars::_serde_json::to_string;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Error {
    pub code: u8,
    pub r#type: String,
    pub context: Vec<String>,
    pub verbose: String
}

impl Error {
    pub fn to_error(&self) -> StdError {
        StdError::generic_err(self.to_string())
    }

    pub fn to_string(&self) -> String {
        to_string(&self).unwrap_or("".to_string())
    }
}

pub trait CodeType {
    fn to_type(&self) -> String;
}