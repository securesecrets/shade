#[cfg(test)]
pub mod tests {
    use crate::handle::inverse_normalizer;
    use cosmwasm_std::{Binary, HumanAddr, Uint128};
    use flexible_permits::{
        permit::bech32_to_canonical,
        transaction::{PermitSignature, PubKey},
    };
    use shade_protocol::airdrop::account::{AddressProofMsg, AddressProofPermit, FillerMsg};
    use shade_protocol::utils::math::{div, mult};

    #[test]
    fn decay_factor() {
        assert_eq!(
            Uint128(50),
            Uint128(100) * inverse_normalizer(100, 200, 300)
        );

        assert_eq!(Uint128(25), Uint128(100) * inverse_normalizer(0, 75, 100));
    }

    // #[test]
    // fn secret_signature() {
    //     let permit = AddressProofPermit {
    //         params: AddressProofMsg{
    //             address: HumanAddr("secret19q7h2zy8mgesy3r39el5fcm986nxqjd7cgylrz".to_string()),
    //             amount: Uint128(27994412),
    //             contract: HumanAddr("secret17q23878cx2pmjn8cp7sqhylqfpvdw9r8p5q8um".to_string()),
    //             index: 11,
    //             key: "account-creation-permit".to_string()
    //         },
    //         chain_id: Some("pulsar-2".to_string()),
    //         signature: PermitSignature {
    //             pub_key: PubKey {
    //                 r#type: "tendermint/PubKeySecp256k1".to_string(),
    //                 value: Binary::from_base64(
    //                     "A2uZZ02iy/QhPZ0s6WO8HTEfNZEnt5o5PsQ34WHmQFPK")
    //                     .expect("Base 64 invalid")
    //             },
    //             signature: Binary::from_base64(
    //                 "bK+ns5SrA7JeFtHlwt+aLU6wB4hgebTMgdNfTfbRtS8TQx1xztFsLoRKa1rqGKBSobVqftGHuIN0s/6CgY1Gxw==")
    //                 .expect("Base 64 invalid")
    //         }
    //     };
    // 
    //     let permit_addr = permit.validate().expect("Signature validation failed");
    //     assert_eq!(
    //         permit_addr.as_canonical(),
    //         bech32_to_canonical(permit.params.address.as_str())
    //     );
    //     assert_ne!(
    //         permit_addr.as_canonical(),
    //         bech32_to_canonical("secret17q23878cx2pmjn8cp7sqhylqfpvdw9r8p5q8um")
    //     );
    // }
    // 
    // #[test]
    // fn cosmos_signature() {
    //     let permit = AddressProofPermit {
    //         params: AddressProofMsg{
    //             address: HumanAddr("cosmos1lj5vh5y8yp4a97jmfwpd98lsg0tf5lsqgnnhq3".to_string()),
    //             amount: Uint128(123752075),
    //             contract: HumanAddr("secret17q23878cx2pmjn8cp7sqhylqfpvdw9r8p5q8um".to_string()),
    //             index: 6,
    //             key: "account-creation-permit".to_string()
    //         },
    //         chain_id: Some("cosmoshub-4".to_string()),
    //         signature: PermitSignature {
    //             pub_key: PubKey {
    //                 r#type: "tendermint/PubKeySecp256k1".to_string(),
    //                 value: Binary::from_base64(
    //                     "AqcyBLqPn7QnOctkK9i9KhnhD0aHA03+LppvNTCdZ1wK")
    //                     .expect("Base 64 invalid")
    //             },
    //             signature: Binary::from_base64(
    //                 "IrJPk51qu1X2w3OvCOgEIdM8zBRi379TAYLLh3aCmB8LbNaFbycgtVwtqa4jGGF2jhnkzZCxObk3Y4OMeId+4A==")
    //                 .expect("Base 64 invalid")
    //         }
    //     };
    // 
    //     let permit_addr = permit.validate().expect("Signature validation failed");
    //     assert_eq!(
    //         permit_addr.as_canonical(),
    //         bech32_to_canonical(permit.params.address.as_str())
    //     );
    //     assert_ne!(
    //         permit_addr.as_canonical(),
    //         bech32_to_canonical("cosmos1ceqk06xpqrq45melc9f8khae0fwaa5y5w0gz6x")
    //     );
    // }

    #[test]
    fn ledger_terra_signature() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("bombay-12".to_string()),
            sequence: Some(Uint128(0)),
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "A50CTeVnMYyZGh7K4x4NtdfG1H1oicog6lEoPMi65IK2").unwrap()),
                signature: Binary::from_base64(
                    "75RcVHa/SW1WyjcFMkhZ63+D4ccxffchLvJPyURmtaskA8CPj+y6JSrpuRhxMC+1hdjSJC3c0IeJVbDIRapxPg==").unwrap(),
            },
            account_number: Some(Uint128(203289)),
            memo: Some("b64Encoded".to_string())
        };

        let addr = permit.validate(Some("wasm/MsgExecuteContract".to_string())).expect("Signature validation failed");;
        assert_eq!(addr.as_canonical(), bech32_to_canonical("terra1m79yd3jh97vz4tqu0m8g49gfl7qmknhh23kac5"));
        assert_ne!(addr.as_canonical(), bech32_to_canonical("terra19m2zgdyuq0crpww00jc2a9k70ut944dum53p7x"));

        permit.memo = Some("OtherMemo".to_string());

        assert!(permit.validate(Some("wasm/MsgExecuteContract".to_string())).is_err())
    }

    #[test]
    fn claim_query() {
        assert_eq!(
            Uint128(300),
            mult(div(Uint128(345), Uint128(100)).unwrap(), Uint128(100))
        )
    }

    #[test]
    fn claim_query_odd_multiple() {
        assert_eq!(
            Uint128(13475),
            mult(div(Uint128(13480), Uint128(7)).unwrap(), Uint128(7))
        )
    }

    #[test]
    fn claim_query_under_step() {
        assert_eq!(
            Uint128(0),
            mult(div(Uint128(200), Uint128(1000)).unwrap(), Uint128(1000))
        )
    }
}
