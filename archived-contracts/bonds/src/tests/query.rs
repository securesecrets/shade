use shade_protocol::c_std::{testing::*, Binary, Addr};
use shade_protocol::fadroma::core::ContractLink;
use shade_protocol::fadroma::ensemble::ContractEnsemble;
use shade_protocol::contract_interfaces::{
    bonds,
    query_auth::{self, PermitData, QueryPermit},
    snip20::helpers::Snip20Asset,
};

use shade_protocol::query_authentication::transaction::{PermitSignature, PubKey};

use shade_protocol::c_std::Uint128;

pub fn query_no_opps(chain: &mut ContractEnsemble, bonds: &ContractLink<Addr>) -> () {
    let msg = bonds::QueryMsg::BondOpportunities {};

    let query: bonds::QueryAnswer = chain.query(bonds.address.clone(), &msg).unwrap();

    match query {
        bonds::QueryAnswer::BondOpportunities { bond_opportunities } => {
            assert_eq!(bond_opportunities, vec![]);
        }
        _ => assert!(false),
    }
}

pub fn query_opp_parameters(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<Addr>,
    issuance_limit: Option<Uint128>,
    amount_issued: Option<Uint128>,
    deposit_denom: Option<Snip20Asset>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    max_accepted_deposit_price: Option<Uint128>,
    err_deposit_price: Option<Uint128>,
    minting_bond: Option<bool>,
) -> () {
    let query: bonds::QueryAnswer = chain
        .query(
            bonds.address.clone(),
            &bonds::QueryMsg::BondOpportunities {},
        )
        .unwrap();

    match query {
        bonds::QueryAnswer::BondOpportunities {
            bond_opportunities, ..
        } => {
            if issuance_limit.is_some() {
                assert_eq!(
                    bond_opportunities[0].issuance_limit,
                    issuance_limit.unwrap()
                )
            }
            if amount_issued.is_some() {
                assert_eq!(bond_opportunities[0].amount_issued, amount_issued.unwrap())
            }
            if deposit_denom.is_some() {
                assert_eq!(bond_opportunities[0].deposit_denom, deposit_denom.unwrap())
            }
            if start_time.is_some() {
                assert_eq!(bond_opportunities[0].start_time, start_time.unwrap())
            }
            if end_time.is_some() {
                assert_eq!(bond_opportunities[0].end_time, end_time.unwrap())
            }
            if bonding_period.is_some() {
                assert_eq!(
                    bond_opportunities[0].bonding_period,
                    bonding_period.unwrap()
                )
            }
            if discount.is_some() {
                assert_eq!(bond_opportunities[0].discount, discount.unwrap())
            }
            if max_accepted_deposit_price.is_some() {
                assert_eq!(
                    bond_opportunities[0].max_accepted_deposit_price,
                    max_accepted_deposit_price.unwrap()
                )
            }
            if err_deposit_price.is_some() {
                assert_eq!(
                    bond_opportunities[0].err_deposit_price,
                    err_deposit_price.unwrap()
                )
            }
            if minting_bond.is_some() {
                assert_eq!(bond_opportunities[0].minting_bond, minting_bond.unwrap())
            }
        }
        _ => assert!(false),
    };
}

pub fn query_acccount_parameters(
    chain: &mut ContractEnsemble,
    bonds: &ContractLink<Addr>,
    query_auth: &ContractLink<Addr>,
    _sender: &str,
    deposit_denom: Option<Snip20Asset>,
    end_time: Option<u64>,
    deposit_amount: Option<Uint128>,
    deposit_price: Option<Uint128>,
    claim_amount: Option<Uint128>,
    claim_price: Option<Uint128>,
    discount: Option<Uint128>,
    discount_price: Option<Uint128>,
) -> () {
    let permit = get_permit();

    let deps = mock_dependencies(20, &[]);

    // Confirm that the permit is valid
    assert!(permit.clone().validate(&deps.api, None).is_ok());

    let _query: query_auth::QueryAnswer = chain
        .query(
            query_auth.address.clone(),
            &query_auth::QueryMsg::ValidatePermit {
                permit: permit.clone(),
            },
        )
        .unwrap();

    let query: bonds::QueryAnswer = chain
        .query(bonds.address.clone(), &bonds::QueryMsg::Account { permit })
        .unwrap();

    match query {
        bonds::QueryAnswer::Account { pending_bonds, .. } => {
            if deposit_denom.is_some() {
                assert_eq!(pending_bonds[0].deposit_denom, deposit_denom.unwrap())
            }
            if end_time.is_some() {
                assert_eq!(pending_bonds[0].end_time, end_time.unwrap())
            }
            if deposit_price.is_some() {
                assert_eq!(pending_bonds[0].deposit_price, deposit_price.unwrap())
            }
            if deposit_amount.is_some() {
                assert_eq!(pending_bonds[0].deposit_amount, deposit_amount.unwrap())
            }
            if claim_amount.is_some() {
                assert_eq!(pending_bonds[0].claim_amount, claim_amount.unwrap())
            }
            if claim_price.is_some() {
                assert_eq!(pending_bonds[0].claim_price, claim_price.unwrap())
            }
            if discount.is_some() {
                assert_eq!(pending_bonds[0].discount, discount.unwrap())
            }
            if discount_price.is_some() {
                assert_eq!(pending_bonds[0].discount_price, discount_price.unwrap())
            }
        }
        _ => assert!(false),
    };
}

fn get_permit() -> QueryPermit {
    QueryPermit {
        params: PermitData {
            key: "key".to_string(),
            data: Binary::from_base64("c29tZSBzdHJpbmc=").unwrap()
        },
        signature: PermitSignature {
            pub_key: PubKey::new(
                Binary::from_base64(
                    "A9NjbriiP7OXCpoTov9ox/35+h5k0y1K0qCY/B09YzAP"
                ).unwrap()
            ),
            signature: Binary::from_base64(
                "XRzykrPmMs0ZhksNXX+eU0TM21fYBZXZogr5wYZGGy11t2ntfySuQNQJEw6D4QKvPsiU9gYMsQ259dOzMZNAEg=="
            ).unwrap()
        },
        account_number: None,
        chain_id: Some(String::from("chain")),
        sequence: None,
        memo: None
    }
}
