use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub type CodeId = u64;
pub type CodeHash = String;

/// Info needed to instantiate a contract.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ContractInstantiationInfo {
    pub code_hash: CodeHash,
    pub id: CodeId,
}

// Disregard code hash because it is case insensitive.
// Converting to the same case first and the comparing is unnecessary
// as providing the wrong code hash when calling a contract will result
// in an error regardless and we have no way of checking that here.
impl PartialEq for ContractInstantiationInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084"
                    .into()
            },
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084"
                    .into()
            }
        );

        assert_eq!(
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084"
                    .into()
            },
            ContractInstantiationInfo {
                id: 1,
                code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084"
                    .into()
            }
        );

        assert_ne!(
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084"
                    .into()
            },
            ContractInstantiationInfo {
                id: 2,
                code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084"
                    .into()
            }
        );
    }
}
