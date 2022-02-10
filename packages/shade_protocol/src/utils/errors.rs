use cosmwasm_std::StdError;
use schemars::_serde_json::to_string;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

macro_rules! impl_into_u8 {
    ($error:ident) => {
        impl From<$error> for u8 {
            fn from(err: $error) -> u8 {
                err as _
            }
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Error<T: CodeType> {
    pub code: u8,
    pub r#type: T,
    pub context: Vec<String>,
    pub verbose: String
}

impl<T: CodeType + Serialize> Error<T> {
    pub fn to_error(&self) -> StdError {
        StdError::generic_err(self.to_string())
    }
    pub fn to_string(&self) -> String {
        to_string(&self).unwrap_or("".to_string())
    }
    pub fn from_code(code: T, context: Vec<&str>) -> Self {
        let verbose = code.to_verbose(&context);
        Self {
            code: code.to_code(),
            r#type: code,
            context: context.iter().map(|s| s.to_string()).collect(),
            verbose
        }
    }
}

pub trait CodeType: Into<u8> + Clone {
    fn to_code(&self) -> u8 {
        self.clone().into()
    }
    fn to_verbose(&self, context: &Vec<&str>) -> String;
    fn build_string(verbose: &str, context: &Vec<&str>) -> String {
        let mut msg = verbose.to_string();
        for arg in context.iter() {
            msg = msg.replacen("{}", arg, 1);
        }
        msg
    }
}

#[cfg(test)]
pub mod tests {
    use cosmwasm_std::StdError;
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;
    use crate::utils::errors::{Error, CodeType};

    #[derive(Serialize, Deserialize, Copy, Clone, Debug, JsonSchema)]
    #[repr(u8)]
    #[serde(rename_all = "snake_case")]
    enum TestCode {
        Error1,
        Error2,
        Error3
    }

    impl_into_u8!(TestCode);

    impl CodeType for TestCode {
        fn to_verbose(&self, context: &Vec<&str>) -> String {
            match self {
                TestCode::Error1 => TestCode::build_string("Error", context),
                TestCode::Error2 => TestCode::build_string("Broke in {}", context),
                TestCode::Error3 => TestCode::build_string("Expecting {} but got {}", context),
            }
        }
    }

    // Because of set variables, you could implement something like this

    fn error_1() -> StdError {
        Error::from_code(TestCode::Error1, vec![]).to_error()
    }

    fn error_2(context: &[&str; 1]) -> StdError {
        Error::from_code(TestCode::Error2, context.to_vec()).to_error()
    }

    fn error_3(context: &[&str; 2]) -> StdError {
        Error::from_code(TestCode::Error3, context.to_vec()).to_error()
    }

    #[test]
    fn build_string() {
        assert_eq!(TestCode::build_string("Test string {}", &vec!["arg"]), "Test string arg".to_string())
    }

    #[test]
    fn to_code() {
        let code1 = TestCode::Error1;
        assert_eq!(code1.to_code(), 0);

        let code2 = TestCode::Error2;
        assert_eq!(code2.to_code(), 1);

        let code3 = TestCode::Error3;
        assert_eq!(code3.to_code(), 2);
    }

    #[test]
    fn to_verbose() {

    }

    #[test]
    fn from_code() {

    }

    #[test]
    fn to_string() {

    }

    #[test]
    fn to_error() {

    }

    #[test]
    fn helpers() {

    }

}