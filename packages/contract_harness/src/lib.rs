

#[cfg(not(target_arch = "wasm32"))]
pub mod harness;
#[cfg(not(target_arch = "wasm32"))]
pub mod harness_macro;

#[cfg(not(target_arch = "wasm32"))]
pub mod assertions {
    use cosmwasm_std::{StdError, StdResult};
    use shade_protocol::fadroma::ensemble::ExecuteResponse;

    use std::error::Error;
    use std::panic::Location;

    // New error type encapsulating the original error and location data.
    #[derive(Debug, Clone)]
    struct LocatedError<E: Error + 'static> {
        inner: E,
        location: &'static Location<'static>,
    }

    impl<E: Error + 'static> Error for LocatedError<E> {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.inner)
        }
    }

    impl<E: Error + 'static> std::fmt::Display for LocatedError<E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}, {}", self.inner, self.location)
        }
    }

    impl From<StdError> for LocatedError<StdError> {
        #[track_caller]
        fn from(err: StdError) -> Self {
            LocatedError {
                inner: err,
                location: std::panic::Location::caller(),
            }
        }
    }

    fn locate_execute_error(
        res: StdResult<ExecuteResponse>
    ) -> Result<ExecuteResponse, LocatedError<StdError>> {
        match res {
            Ok(res) => Ok(res),
            Err(err) => Err(LocatedError::from(err))
        }
    }

    // Asserts that the execute is correct, if not it will print the error
    pub fn assert_execute(res: StdResult<ExecuteResponse>) -> ExecuteResponse {
        match locate_execute_error(res) {
            Ok(res) => res,
            Err(err) => {
                assert!(false, "{}", err);
                // Doing this so compiler will let this slide
                panic!()
            }
        }
    }
}
