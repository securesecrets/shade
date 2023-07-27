use crate::c_std::StdError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Response, StdResult};
use schemars::_serde_json::to_string;
use serde::Serialize;

// TODO: make another that auto imports

/// Generates the errors
/// Macro takes in an array of error name, error msg and error function
#[macro_export]
macro_rules! errors {
    ($Target:tt; $($EnumError:ident, $VerboseError:tt, $Function:ident),+) => {
        use crate::{
            c_std::StdError,
            generate_errors,
            impl_into_u8,
            utils::errors::{build_string, CodeType, DetailedError},
        };
        use cosmwasm_schema::cw_serde;

        generate_errors!($Target; $($EnumError, $VerboseError, $Function),+);
    }
}

#[macro_export]
macro_rules! generate_errors {
    ($Target:tt; $($EnumError:ident, $VerboseError:tt, $Function:ident),+) => {
        #[cw_serde]
        #[repr(u8)]
        pub enum Error { $($EnumError,)+ }
        impl_into_u8!(Error);

        impl CodeType for Error {
            fn to_verbose(&self, context: &Vec<&str>) -> String {
                match self {
                    $(
                    Error::$EnumError => {
                        build_string($VerboseError, context)
                    }
                    )+
                }
            }
        }

        const TARGET: &str = $Target ;

        impl Error {
            $(
            pub fn $Function(args: Vec<&str>) -> StdError {
                DetailedError::from_code(TARGET, Error::$EnumError, args).to_error()
            }
            )+
        }

    };
}

#[macro_export]
macro_rules! impl_into_u8 {
    ($error:ident) => {
        impl From<$error> for u8 {
            fn from(err: $error) -> u8 {
                err as _
            }
        }
    };
}

#[cw_serde]
pub struct DetailedError<T: CodeType> {
    pub target: String,
    pub code: u8,
    pub r#type: T,
    pub context: Vec<String>,
    pub verbose: String,
}

impl<T: CodeType + Serialize> DetailedError<T> {
    pub fn to_full_error(&self) -> StdResult<Response> {
        Err(self.to_error())
    }

    pub fn to_error(&self) -> StdError {
        StdError::generic_err(self.to_string())
    }

    pub fn to_string(&self) -> String {
        to_string(&self).unwrap_or("".to_string())
    }

    pub fn from_code(target: &str, code: T, context: Vec<&str>) -> Self {
        let verbose = code.to_verbose(&context);
        Self {
            target: target.to_string(),
            code: code.to_code(),
            r#type: code,
            context: context.iter().map(|s| s.to_string()).collect(),
            verbose,
        }
    }
}

pub fn build_string(verbose: &str, context: &Vec<&str>) -> String {
    let mut msg = verbose.to_string();
    for arg in context.iter() {
        msg = msg.replacen("{}", arg, 1);
    }
    msg
}

pub trait CodeType: Into<u8> + Clone {
    fn to_code(&self) -> u8 {
        self.clone().into()
    }
    fn to_verbose(&self, context: &Vec<&str>) -> String;
}

#[cfg(test)]
pub mod tests {
    use crate::{
        c_std::{Response, StdError, StdResult},
        utils::errors::{build_string, CodeType, DetailedError},
    };

    use cosmwasm_schema::cw_serde;

    #[cw_serde]
    #[repr(u8)]
    enum TestCode {
        Error1,
        Error2,
        Error3,
    }

    impl_into_u8!(TestCode);

    impl CodeType for TestCode {
        fn to_verbose(&self, context: &Vec<&str>) -> String {
            match self {
                TestCode::Error1 => build_string("Error", context),
                TestCode::Error2 => build_string("Broke in {}", context),
                TestCode::Error3 => build_string("Expecting {} but got {}", context),
            }
        }
    }

    // Because of set variables, you could implement something like this

    fn error_1() -> StdError {
        DetailedError::from_code("contract", TestCode::Error1, vec![]).to_error()
    }

    fn error_2(context: &[&str; 1]) -> StdError {
        DetailedError::from_code("contract", TestCode::Error2, context.to_vec()).to_error()
    }

    fn error_3(context: &[&str; 2]) -> StdError {
        DetailedError::from_code("contract", TestCode::Error3, context.to_vec()).to_error()
    }

    #[test]
    fn string_builder() {
        assert_eq!(
            build_string("Test string {}", &vec!["arg"]),
            "Test string arg".to_string()
        )
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
        assert_eq!(TestCode::Error1.to_verbose(&vec![]), "Error".to_string());
        assert_eq!(
            TestCode::Error2.to_verbose(&vec!["function"]),
            "Broke in function".to_string()
        );
        assert_eq!(
            TestCode::Error3.to_verbose(&vec!["address", "amount"]),
            "Expecting address but got amount".to_string()
        );
    }

    #[test]
    fn from_code() {
        let err1 = DetailedError::from_code("contract", TestCode::Error1, vec![]);
        assert_eq!(err1.code, 0);
        assert_eq!(err1.r#type, TestCode::Error1);
        let empty: Vec<String> = vec![];
        assert_eq!(err1.context, empty);
        assert_eq!(err1.verbose, "Error".to_string());

        let err2 = DetailedError::from_code("contract", TestCode::Error2, vec!["function"]);
        assert_eq!(err2.code, 1);
        assert_eq!(err2.r#type, TestCode::Error2);
        assert_eq!(err2.context, vec!["function".to_string()]);
        assert_eq!(err2.verbose, "Broke in function".to_string());

        let err3 =
            DetailedError::from_code("contract", TestCode::Error3, vec!["address", "amount"]);
        assert_eq!(err3.code, 2);
        assert_eq!(err3.r#type, TestCode::Error3);
        assert_eq!(err3.context, vec![
            "address".to_string(),
            "amount".to_string()
        ]);
        assert_eq!(err3.verbose, "Expecting address but got amount".to_string());
    }

    #[test]
    fn to_string() {
        assert_eq!(DetailedError::from_code("contract", TestCode::Error1, vec![]).to_string(),
                   "{\"target\":\"contract\",\"code\":0,\"type\":\"error1\",\"context\":[],\"verbose\":\"Error\"}".to_string());
        assert_eq!(DetailedError::from_code("contract", TestCode::Error2, vec!["function"]).to_string(),
                   "{\"target\":\"contract\",\"code\":1,\"type\":\"error2\",\"context\":[\"function\"],\"verbose\":\"Broke in function\"}".to_string());
        assert_eq!(DetailedError::from_code("contract", TestCode::Error3, vec!["address", "amount"]).to_string(),
                   "{\"target\":\"contract\",\"code\":2,\"type\":\"error3\",\"context\":[\"address\",\"amount\"],\"verbose\":\"Expecting address but got amount\"}".to_string());
    }

    #[test]
    fn to_error() {
        let err1 = DetailedError::from_code("contract", TestCode::Error1, vec![]).to_error();
        match err1 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":0,\"type\":\"error1\",\"context\":[],\"verbose\":\"Error\"}".to_string()),
            _ => assert!(false)
        }

        let err2 =
            DetailedError::from_code("contract", TestCode::Error2, vec!["function"]).to_error();
        match err2 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":1,\"type\":\"error2\",\"context\":[\"function\"],\"verbose\":\"Broke in function\"}".to_string()),
            _ => assert!(false)
        }

        let err3 =
            DetailedError::from_code("contract", TestCode::Error3, vec!["address", "amount"])
                .to_error();
        match err3 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":2,\"type\":\"error3\",\"context\":[\"address\",\"amount\"],\"verbose\":\"Expecting address but got amount\"}".to_string()),
            _ => assert!(false)
        }
    }

    #[test]
    fn helpers() {
        let err1 = error_1();
        match err1 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":0,\"type\":\"error1\",\"context\":[],\"verbose\":\"Error\"}".to_string()),
                _ => assert!(false)
        }

        let err2 = error_2(&["function"]);
        match err2 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":1,\"type\":\"error2\",\"context\":[\"function\"],\"verbose\":\"Broke in function\"}".to_string()),
            _ => assert!(false)
        }

        let err3 = error_3(&["address", "amount"]);
        match err3 {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "{\"target\":\"contract\",\"code\":2,\"type\":\"error3\",\"context\":[\"address\",\"amount\"],\"verbose\":\"Expecting address but got amount\"}".to_string()),
            _ => assert!(false)
        }
    }

    generate_errors!("test"; 
        Test1, "Verb1", Test1Func,
        Test2, "Ver2", Test2Func,
        Test3, "Verb3", Test3Func);

    #[test]
    fn macro_errors() {
        let test = Error::Test2;

        let test1 = Error::Test1Func(vec![]);
    }
}
