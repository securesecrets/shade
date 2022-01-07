#[cfg(test)]
pub mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr, Uint128};
    use shade_protocol::airdrop::account::{AddressProofMsg, AddressProofPermit};
    use shade_protocol::airdrop::claim_info::RequiredTask;
    use shade_protocol::airdrop::InitMsg;
    use shade_protocol::asset::Contract;
    use shade_protocol::signature::{bech32_to_canonical, pubkey_to_account};
    use shade_protocol::signature::transaction::{PermitSignature, PubKey};
    use crate::contract::init;
    use crate::handle::inverse_normalizer;

    #[test]
    fn decay_factor() {
        assert_eq!(Uint128(50), Uint128(100) * inverse_normalizer(100, 200, 300));

        assert_eq!(Uint128(25), Uint128(100) * inverse_normalizer(0, 75, 100));
    }

    #[test]
    fn secret_signature() {
        let permit = AddressProofPermit {
            params: AddressProofMsg{
                address: HumanAddr("secret19q7h2zy8mgesy3r39el5fcm986nxqjd7cgylrz".to_string()),
                amount: Uint128(27994412),
                contract: HumanAddr("secret17q23878cx2pmjn8cp7sqhylqfpvdw9r8p5q8um".to_string()),
                index: 11,
                key: "account-creation-permit".to_string()
            },
            chain_id: "pulsar-2".to_string(),
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64(
                        "A2uZZ02iy/QhPZ0s6WO8HTEfNZEnt5o5PsQ34WHmQFPK")
                        .expect("Base 64 invalid")
                },
                signature: Binary::from_base64(
                    "bK+ns5SrA7JeFtHlwt+aLU6wB4hgebTMgdNfTfbRtS8TQx1xztFsLoRKa1rqGKBSobVqftGHuIN0s/6CgY1Gxw==")
                    .expect("Base 64 invalid")
            }
        };

        let permit_addr = permit.validate().expect("Signature validation failed");
    }

    #[test]
    fn cosmos_signature() {
        let permit = AddressProofPermit {
            params: AddressProofMsg{
                address: HumanAddr("cosmos1lj5vh5y8yp4a97jmfwpd98lsg0tf5lsqgnnhq3".to_string()),
                amount: Uint128(123752075),
                contract: HumanAddr("secret17q23878cx2pmjn8cp7sqhylqfpvdw9r8p5q8um".to_string()),
                index: 6,
                key: "account-creation-permit".to_string()
            },
            chain_id: "cosmoshub-4".to_string(),
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64(
                        "AqcyBLqPn7QnOctkK9i9KhnhD0aHA03+LppvNTCdZ1wK")
                        .expect("Base 64 invalid")
                },
                signature: Binary::from_base64(
                    "IrJPk51qu1X2w3OvCOgEIdM8zBRi379TAYLLh3aCmB8LbNaFbycgtVwtqa4jGGF2jhnkzZCxObk3Y4OMeId+4A==")
                    .expect("Base 64 invalid")
            }
        };

        let permit_addr = permit.validate().expect("Signature validation failed");
        assert_eq!(permit_addr, bech32_to_canonical(permit.params.address.clone().as_str()));
        assert_ne!(permit_addr, bech32_to_canonical("cosmos1ceqk06xpqrq45melc9f8khae0fwaa5y5w0gz6x"));
    }

    #[test]
    fn terra_signature() {
        let permit = AddressProofPermit {
            params: AddressProofMsg{
                address: HumanAddr("terra1vypeq4lqlsh9k443ghf04uexv9xlzxqlxnrjhl".to_string()),
                amount: Uint128(112362871),
                contract: HumanAddr("secret17q23878cx2pmjn8cp7sqhylqfpvdw9r8p5q8um".to_string()),
                index: 0,
                key: "account-creation-permit".to_string()
            },
            chain_id: "columbus-5".to_string(),
            signature: PermitSignature {
                pub_key: PubKey {
                    r#type: "tendermint/PubKeySecp256k1".to_string(),
                    value: Binary::from_base64(
                        "A7DF52PBYi26Mgi8zhWCc6IzKTgy/DnNSj5oxlFNT8XU")
                        .expect("Base 64 invalid")
                },
                signature: Binary::from_base64(
                    "/DFPeocGvP9m/k4h0RxkTQkH5hm7YgjKtBwsly/GdcgN7UPz4ZfZo8xSzhudMbxR1PyQjcNdKLL5IJvYBQCWBQ==")
                    .expect("Base 64 invalid")
            }
        };

        let permit_addr = permit.validate().expect("Signature validation failed");
        assert_eq!(permit_addr, bech32_to_canonical(permit.params.address.clone().as_str()));
        assert_ne!(permit_addr, bech32_to_canonical("terra19m2zgdyuq0crpww00jc2a9k70ut944dum53p7x"));
    }


}