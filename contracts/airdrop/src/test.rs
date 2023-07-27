#[cfg(test)]
pub mod tests {
    use crate::handle::inverse_normalizer;
    use shade_protocol::{
        airdrop::account::{AddressProofMsg, AddressProofPermit, FillerMsg},
        c_std::{from_binary, testing::mock_dependencies, Addr, Binary, Uint128},
        query_authentication::{
            permit::bech32_to_canonical,
            transaction::{PermitSignature, PubKey},
        },
    };

    #[test]
    fn decay_factor() {
        assert_eq!(
            Uint128::new(50u128),
            Uint128::new(100u128) * inverse_normalizer(100, 200, 300)
        );

        assert_eq!(
            Uint128::new(25u128),
            Uint128::new(100u128) * inverse_normalizer(0, 75, 100)
        );
    }

    const MSGTYPE: &str = "wasm/MsgExecuteContract";

    #[test]
    fn terra_station_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("columbus-5".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "Ar7aIv8k6Rm7ugLBAHShtRWmZ/CDgvwXYOc8Ffycwggc").unwrap()),
                signature: Binary::from_base64(
                    "MM1UOheGCYX0Cb3r8zVhyZyWk/qIY61yqiDP53//31cjkd7G5FfEki+JC91kBRYCnt9NlI7gjnY8ZcJauDH3FA==").unwrap(),
            },
            account_number: Some(Uint128::new(3441602u128).into()),
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("terra17dhvxnwzazszgtuc498qsudh7zq945qh29gj4e")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("terra19m2zgdyuq0crpww00jc2a9k70ut944dum53p7x")
        );

        permit.memo = Some("OtherMemo".to_string());

        // NOTE: New SN broke unit testing
        // assert!(
        //     permit
        //         .validate(&deps.api, Some("wasm/MsgExecuteContract".to_string()))
        //         .is_err()
        // )
    }

    #[test]
    fn terra_station_non_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("columbus-5".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "A8r22cTiywZYSoWR5DnmAeP1jPDF3CLVKJe1QGorv9cM").unwrap()),
                signature: Binary::from_base64(
                    "xhU2JkJDWO/eZEeJVp8vo1rNAK7H7G2uDucZAjAhfVRjLHHX7C+16dwQzr0Jmd2DdZHAJZNkGhGb5nucicN1TA==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("terra1j8wupj3kpclp98dgg4j5am44kjykx6uztjttyr")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("terra1ns69jhkjg5wmcgf8w8ecewnpca7sezyhvg0a29")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn keplr_terra_non_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("columbus-5".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "AyGKvc3OCs/pg7unFCJgKjtqiLYRACeR4ZU0f8UVDFbM").unwrap()),
                signature: Binary::from_base64(
                    "fbgFeYUsAjI2CB2dwaqttolFE1wx/3MXbNWYKicJj20mV3marS4zz+k5aCKsYlv4HSd9NYxl4deuhasMKndB2w==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("terra18xg6g5yfzflnt8v45r2yndnydhg2vndvzsv3rn")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("terra1ns69jhkjg5wmcgf8w8ecewnpca7sezyhvg0a29")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn keplr_terra_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("columbus-5".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "AqjDFVbY+znM1F5XCDuaca0JT0uAdd3QyuHt04j9k0DB").unwrap()),
                signature: Binary::from_base64(
                    "1kwDnlsltqgj8fqbohs0MMgEWiRMUmznM98ofOranBAe5f8Ja1tZCKmh5miPkgC6KoUdOam7BvBjuFhM1q0rBA==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("terra1ns69jhkjg5wmcgf8w8ecewnpca7sezyhvg0a29")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("terra18xg6g5yfzflnt8v45r2yndnydhg2vndvzsv3rn")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn keplr_sn_non_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("secret-4".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "A2uZZ02iy/QhPZ0s6WO8HTEfNZEnt5o5PsQ34WHmQFPK").unwrap()),
                signature: Binary::from_base64(
                    "s80mH5OuZCudS20d0k73evWx5xGrC2l3uubQjIkukT4L5mcgsepDIq9d1YpAJwiUEitaHFOGy42MfHZJVY1LdA==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("secret19q7h2zy8mgesy3r39el5fcm986nxqjd7cgylrz")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("secret1ns69jhkjg5wmcgf8w8ecewnpca7sezyhgfp54e")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn keplr_sn_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("secret-4".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "AqjDFVbY+znM1F5XCDuaca0JT0uAdd3QyuHt04j9k0DB").unwrap()),
                signature: Binary::from_base64(
                    "snd8k5nWAAVoUytxKZt1FCUNQNXLAQpBlF7h4YGbTmx3S5+rqaZnM2bKq1ifCvErz/pdeE7B/s+WsGLdQRpzoA==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("secret1ns69jhkjg5wmcgf8w8ecewnpca7sezyhgfp54e")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("secret19q7h2zy8mgesy3r39el5fcm986nxqjd7cgylrz")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn keplr_cosmos_non_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("cosmoshub-4".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "AqcyBLqPn7QnOctkK9i9KhnhD0aHA03+LppvNTCdZ1wK").unwrap()),
                signature: Binary::from_base64(
                    "KLwotev7wnbj2VGBxbyTfIrRn/1vQY3x3I7BAUhu4FIC6OHVXqxIl/lclgdBWksnr32ULVfz8u78OqEbaePRZQ==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("cosmos1lj5vh5y8yp4a97jmfwpd98lsg0tf5lsqgnnhq3")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("cosmos1ns69jhkjg5wmcgf8w8ecewnpca7sezyh2v4ag9")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn keplr_cosmos_ledger() {
        let mut permit = AddressProofPermit {
            params: FillerMsg::default(),
            chain_id: Some("cosmoshub-4".to_string()),
            sequence: None,
            signature: PermitSignature {
                pub_key: PubKey::new(Binary::from_base64(
                    "AqjDFVbY+znM1F5XCDuaca0JT0uAdd3QyuHt04j9k0DB").unwrap()),
                signature: Binary::from_base64(
                    "h/RpG1eKzN03oId0GvN7TSxoHOUibjmqPEQ1E+ZWh+BvghPL99lBj4L3BKpjjsaRtXX3lexO7ztafLKBVtq4xA==").unwrap(),
            },
            account_number: None,
            memo: Some("eyJhbW91bnQiOiIxMDAwMDAwMCIsImluZGV4IjoxMCwia2V5IjoiYWNjb3VudC1jcmVhdGlvbi1wZXJtaXQifQ==".to_string())
        };

        let deps = mock_dependencies();
        let addr = permit
            .validate(&deps.api, Some(MSGTYPE.to_string()))
            .expect("Signature validation failed");
        assert_eq!(
            addr.as_canonical(),
            bech32_to_canonical("cosmos1ns69jhkjg5wmcgf8w8ecewnpca7sezyh2v4ag9")
        );
        assert_ne!(
            addr.as_canonical(),
            bech32_to_canonical("cosmos1lj5vh5y8yp4a97jmfwpd98lsg0tf5lsqgnnhq3")
        );

        permit.memo = Some("OtherMemo".to_string());

        // assert!(permit.validate(&deps.api, Some(MSGTYPE.to_string())).is_err())
    }

    #[test]
    fn memo_deserialization() {
        let expected_memo = AddressProofMsg {
            address: Addr::unchecked("secret19q7h2zy8mgesy3r39el5fcm986nxqjd7cgylrz".to_string()),
            amount: Uint128::new(1000000u128),
            contract: Addr::unchecked("secret1sr62lehajgwhdzpmnl65u35rugjrgznh2572mv".to_string()),
            index: 10,
            key: "account-creation-permit".to_string(),
        };

        let deserialized_memo: AddressProofMsg = from_binary(
            &Binary::from_base64(
                &"eyJhZGRyZXNzIjoic2VjcmV0MTlxN2gyenk4bWdlc3kzcjM5ZWw1ZmNtOTg2bnhxamQ3Y2d5bHJ6IiwiYW1vdW50IjoiMTAwMDAwMCIsImNvbnRyYWN0Ijoic2VjcmV0MXNyNjJsZWhhamd3aGR6cG1ubDY1dTM1cnVnanJnem5oMjU3Mm12IiwiaW5kZXgiOjEwLCJrZXkiOiJhY2NvdW50LWNyZWF0aW9uLXBlcm1pdCJ9"
                    .to_string()).unwrap()).unwrap();

        assert_eq!(deserialized_memo, expected_memo)
    }

    #[test]
    fn claim_query() {
        assert_eq!(
            Uint128::new(300u128),
            (Uint128::new(345u128) / Uint128::new(100u128)) * Uint128::new(100u128)
        )
    }

    #[test]
    fn claim_query_odd_multiple() {
        assert_eq!(
            Uint128::new(13475u128),
            (Uint128::new(13480u128) / Uint128::new(7u128)) * Uint128::new(7u128)
        )
    }

    #[test]
    fn claim_query_under_step() {
        assert_eq!(
            Uint128::zero(),
            (Uint128::new(200u128) / Uint128::new(1000u128)) * Uint128::new(1000u128)
        )
    }
}
