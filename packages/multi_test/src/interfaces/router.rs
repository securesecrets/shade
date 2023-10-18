use crate::{
    interfaces::utils::{DeployedContracts, SupportedContracts},
    multi::{admin::init_admin_auth, router::Router},
};
use anyhow::Error;
use shade_protocol::{
    c_std::{to_binary, Addr, Coin, ContractInfo, StdError, StdResult, Uint128},
    liquidity_book::lb_pair::SwapResult,
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

use shadeswap_shared::{
    core::TokenAmount,
    router::{self, Hop},
};

pub fn init(chain: &mut App, sender: &str, contracts: &mut DeployedContracts) -> StdResult<()> {
    let admin_auth = match contracts.get(&SupportedContracts::AdminAuth) {
        Some(admin) => admin.clone(),
        None => {
            let contract = Contract::from(init_admin_auth(chain, &Addr::unchecked(sender)));
            contracts.insert(SupportedContracts::AdminAuth, contract.clone());
            contract
        }
    };

    let router = Contract::from(
        match (router::InitMsg {
            prng_seed: to_binary("password").unwrap(),
            entropy: to_binary("password").unwrap(),
            admin_auth,
            airdrop_address: None,
        }
        .test_init(
            Router::default(),
            chain,
            Addr::unchecked(sender),
            "router",
            &[],
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    contracts.insert(SupportedContracts::Router, router);

    Ok(())
}

pub fn swap_tokens_for_exact_tokens(
    chain: &mut App,
    sender: &str,
    router: &ContractInfo,
    offer: TokenAmount,
    expected_return: Option<Uint128>,
    path: Vec<Hop>,
    recipient: Option<String>,
) -> StdResult<()> {
    let mut funds = Vec::new();
    if offer.token.is_native_token() {
        funds = [Coin {
            denom: offer.token.unique_key(),
            amount: offer.amount,
        }]
        .to_vec();
    }

    match (router::ExecuteMsg::SwapTokensForExact {
        offer,
        expected_return,
        path,
        recipient,
        padding: None,
    }
    .test_exec(&router, chain, Addr::unchecked(sender), &funds))
    {
        Ok(_) => Ok::<(), Error>(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    };

    Ok(())
}

pub fn register_snip20_token(
    chain: &mut App,
    sender: &str,
    router: &ContractInfo,
    snip20_token: &ContractInfo,
) -> Result<(), anyhow::Error> {
    let res = router::ExecuteMsg::RegisterSNIP20Token {
        token_addr: snip20_token.address.to_string(),
        token_code_hash: snip20_token.code_hash.to_string(),
        oracle_key: None,
        padding: None,
    }
    .test_exec(&router, chain, Addr::unchecked(sender), &[]);
    match res {
        Ok(_) => Ok::<(), Error>(()),
        Err(e) => return Err(e),
    };

    Ok(())
}

pub fn query_router_registered_tokens(app: &App, router: &ContractInfo) -> StdResult<Vec<Addr>> {
    let res = router::QueryMsg::RegisteredTokens {}.test_query(router, app)?;
    let tokens = match res {
        router::QueryMsgResponse::RegisteredTokens { tokens } => tokens,
        _ => panic!("Query failed"),
    };
    Ok(tokens)
}

pub fn query_swap_simulation(
    app: &App,
    router: &ContractInfo,
    offer: TokenAmount,
    path: Vec<Hop>,
    exclude_fee: Option<bool>,
) -> StdResult<(Uint128, Uint128, Uint128, SwapResult, String)> {
    let res = router::QueryMsg::SwapSimulation {
        offer,
        path,
        exclude_fee,
    }
    .test_query(router, app)?;
    let tokens = match res {
        router::QueryMsgResponse::SwapSimulation {
            total_fee_amount,
            lp_fee_amount,
            shade_dao_fee_amount,
            result,
            price,
        } => (
            total_fee_amount,
            lp_fee_amount,
            shade_dao_fee_amount,
            result,
            price,
        ),
        _ => panic!("Query failed"),
    };
    Ok(tokens)
}
