// In the test module, import the necessary dependencies

// test_instantiate: Test the instantiation of the contract with valid input and check the initial state.
// test_instantiate_invalid_sender: Test the instantiation of the contract with an unauthorized sender.
// test_approve_for_all: Test the approval for all functionality with valid input.
// test_approve_for_all_already_approved: Test the approval for all functionality when the spender is already approved.
// test_batch_transfer_from_valid: Test the batch transfer from functionality with valid input and check the resulting state.
// test_batch_transfer_from_invalid_lengths: Test the batch transfer from functionality with an unequal number of ids and amounts.
// test_batch_transfer_from_insufficient_funds: Test the batch transfer from functionality with insufficient funds in the sender's account.
// test_batch_transfer_from_not_approved: Test the batch transfer from functionality when the spender is not approved.
// test_mint_valid: Test the mint functionality with valid input and check the resulting state.
// test_mint_unauthorized: Test the mint functionality with an unauthorized sender.
// test_burn_valid: Test the burn functionality with valid input and check the resulting state.
// test_burn_unauthorized: Test the burn functionality with an unauthorized sender.
// test_burn_insufficient_funds: Test the burn functionality with insufficient funds in the owner's account.
// test_burn_insufficient_supply: Test the burn functionality with an insufficient total supply.
// test_query_name: Test the query for the contract name.
// test_query_symbol: Test the query for the contract symbol.
// test_query_decimals: Test the query for the contract decimals.
// test_query_total_supply: Test the query for the total supply of a specific token ID.
// test_query_balance_of: Test the query for the balance of a specific token ID for a given owner.
// test_query_balance_of_batch: Test the query for the balances of multiple token IDs for a list of owners.
// test_query_is_approved_for_all: Test the query for checking if a spender is approved for all tokens of an owner.

#[cfg(test)]
mod tests {
    use crate::error::LBTokenError as Error;

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        BalanceOfBatchResponse, BalanceOfResponse, DecimalsResponse, ExecuteMsg, InstantiateMsg,
        IsApprovedForAllResponse, NameResponse, QueryMsg, SymbolResponse, TotalSupplyResponse,
    };
    use crate::state::{Config, BALANCES, CONFIG, SPENDER_APPROVALS, TOTAL_SUPPLY};

    use anyhow::Ok;
    use anyhow::Result;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{attr, from_binary, Addr, Api, Empty, OwnedDeps, Uint256};

    const SENDER: &str = "sender";
    const RECIPIENT: &str = "recipient";
    const LB_PAIR: &str = "lbpair";
    const NAME: &str = "Token";
    const SYMBOL: &str = "TKN";
    const DECIMALS: u8 = 18;

    // Helper function to instantiate the contract
    fn setup_contract() -> Result<OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>> {
        let mut deps = mock_dependencies();

        let instantiate_msg = InstantiateMsg {
            lb_pair: Addr::unchecked(LB_PAIR),
            name: NAME.into(),
            symbol: SYMBOL.into(),
            decimals: DECIMALS,
        };
        let info = mock_info(SENDER, &[]);
        instantiate(deps.as_mut(), mock_env(), info, instantiate_msg)?;
        Ok(deps)
    }

    #[test]
    fn test_instantiate() -> Result<()> {
        // Instantiate the contract with valid input
        let deps = setup_contract()?;

        // Check the initial state after instantiation
        let config: Config = CONFIG.load(deps.as_ref().storage)?;
        assert_eq!(config.name, NAME);
        assert_eq!(config.symbol, SYMBOL);
        assert_eq!(config.decimals, DECIMALS);
        assert_eq!(
            deps.as_ref().api.addr_humanize(&config.admin)?,
            Addr::unchecked(SENDER)
        );
        assert_eq!(
            deps.as_ref().api.addr_humanize(&config.lb_pair)?,
            Addr::unchecked(LB_PAIR)
        );

        Ok(())
    }

    #[test]
    fn test_approve_for_all_edge_cases() -> Result<()> {
        let mut deps = setup_contract()?;

        let spender = "spender";

        // Case 1: Self approval
        let approve_for_all_msg = ExecuteMsg::ApproveForAll {
            spender: Addr::unchecked(SENDER),
            approved: true,
        };
        let info = mock_info(SENDER, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, approve_for_all_msg);

        match res {
            Err(Error::SelfApproval) => (),
            _ => panic!("Expected error: SelfApproval"),
        }
        // Case 2: Approve the spender for the first time
        let approve_for_all_msg = ExecuteMsg::ApproveForAll {
            spender: Addr::unchecked(spender),
            approved: true,
        };
        let info = mock_info(SENDER, &[]);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            approve_for_all_msg.clone(),
        )?;
        assert_eq!(4, res.attributes.len());
        assert_eq!(res.attributes[0], attr("action", "approve_for_all"));

        // Case 3: Approve the spender again
        let res = execute(deps.as_mut(), mock_env(), info, approve_for_all_msg);
        match res {
            Err(Error::AlreadyApproved {}) => (),
            _ => panic!("Expected error: AlreadyApproved"),
        }

        Ok(())
    }

    #[test]
    fn test_batch_transfer_from_success() -> Result<()> {
        let mut deps = setup_contract()?;

        let from = "addr0000";
        let to = "addr1111";
        let spender = "addr2222";
        let ids = vec![1u32, 2u32, 3u32];
        let amounts = vec![
            Uint256::from(10u128),
            Uint256::from(20u128),
            Uint256::from(30u128),
        ];

        // Set initial balances and approvals
        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let key = (deps.api.addr_canonicalize(from)?, *id);
            BALANCES.insert(&mut deps.storage, &key, amount)?;
        }
        SPENDER_APPROVALS.insert(
            &mut deps.storage,
            &(
                deps.api.addr_canonicalize(from)?,
                deps.api.addr_canonicalize(spender)?,
            ),
            &true,
        )?;

        // Execute the batch transfer
        let info = mock_info(spender, &[]);
        let msg = ExecuteMsg::BatchTransferFrom {
            from: Addr::unchecked(from),
            to: Addr::unchecked(to),
            ids: ids.clone(),
            amounts: amounts.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg)?;

        // Check the balances
        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let key_from: (cosmwasm_std::CanonicalAddr, u32) =
                (deps.api.addr_canonicalize(from)?, *id);
            let balance_from = BALANCES.get(&deps.storage, &key_from).unwrap();
            assert_eq!(Uint256::zero(), balance_from);

            let key_to = (deps.api.addr_canonicalize(to)?, *id);
            let balance_to = BALANCES.get(&deps.storage, &key_to).unwrap();
            assert_eq!(*amount, balance_to);
        }

        Ok(())
    }

    #[test]
    fn test_batch_transfer_from_mismatched_lengths() -> Result<()> {
        let mut deps = setup_contract()?;

        let spender = "addr0000";
        let from = "addr1111";
        let to = "addr2222";
        let ids = vec![1u32, 2u32];
        let amounts = vec![Uint256::from(10u128)];

        let info = mock_info(spender, &[]);
        let msg = ExecuteMsg::BatchTransferFrom {
            from: Addr::unchecked(from),
            to: Addr::unchecked(to),
            ids,
            amounts,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(matches!(res.unwrap_err(), Error::InvalidInput(_)));

        Ok(())
    }

    #[test]
    fn test_batch_transfer_from_spender_not_approved() -> Result<()> {
        let mut deps = setup_contract()?;

        let spender = "addr0000";
        let from = "addr1111";
        let to = "addr2222";
        let ids = vec![1u32];
        let amounts = vec![Uint256::from(10u128)];

        let info = mock_info(spender, &[]);
        let msg = ExecuteMsg::BatchTransferFrom {
            from: Addr::unchecked(from),
            to: Addr::unchecked(to),
            ids,
            amounts,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(matches!(res.unwrap_err(), Error::SpenderNotApproved));

        Ok(())
    }

    #[test]
    fn test_batch_transfer_from_insufficient_funds() -> Result<()> {
        let mut deps = setup_contract()?;

        let spender = "addr0000";
        let from = "addr1111";
        let to = "addr2222";
        let ids = vec![1u32];
        let amounts = vec![Uint256::from(10u128)];

        // Approve the spender
        SPENDER_APPROVALS.insert(
            &mut deps.storage,
            &(
                deps.api.addr_canonicalize(from)?,
                deps.api.addr_canonicalize(spender)?,
            ),
            &true,
        )?;

        let info = mock_info(spender, &[]);
        let msg = ExecuteMsg::BatchTransferFrom {
            from: Addr::unchecked(from),
            to: Addr::unchecked(to),
            ids,
            amounts,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(matches!(res.unwrap_err(), Error::InsufficientFunds));

        Ok(())
    }

    #[test]
    fn test_batch_transfer_from_empty_list() -> Result<()> {
        let mut deps = setup_contract()?;

        let spender = "addr0000";
        let from = "addr1111";
        let to = "addr2222";
        let ids: Vec<u32> = vec![];
        let amounts: Vec<Uint256> = vec![];

        let info = mock_info(spender, &[]);
        let msg = ExecuteMsg::BatchTransferFrom {
            from: Addr::unchecked(from),
            to: Addr::unchecked(to),
            ids,
            amounts,
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(matches!(res.unwrap_err(), Error::InvalidInput(..)));

        Ok(())
    }

    #[test]
    fn test_mint() -> Result<()> {
        let mut deps = setup_contract()?;
        let lb_pair = LB_PAIR;
        let recipient = RECIPIENT;
        let id = 1u32;
        let amount = Uint256::from(100u128);

        // Ensure minting from unauthorized sender fails
        let info = mock_info("addr2222", &[]);
        let msg = ExecuteMsg::Mint {
            recipient: Addr::unchecked(recipient),
            id,
            amount: amount.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(matches!(res.unwrap_err(), Error::Unauthorized));

        // Mint tokens
        let info = mock_info(lb_pair, &[]);
        let msg = ExecuteMsg::Mint {
            recipient: Addr::unchecked(recipient),
            id,
            amount: amount.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg)?;
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "mint");

        // Check recipient's balance
        let recipient_canonical = deps.api.addr_canonicalize(recipient)?;
        let balance = BALANCES
            .get(&deps.storage, &(recipient_canonical, id))
            .unwrap();
        assert_eq!(balance, amount);

        // Check total supply
        let total_supply = TOTAL_SUPPLY.get(&deps.storage, &id).unwrap();
        assert_eq!(total_supply, amount);

        Ok(())
    }

    #[test]
    fn test_burn() -> Result<()> {
        let mut deps = setup_contract()?;
        let lb_pair = LB_PAIR;
        let owner = "addr1111";
        let id = 1u32;
        let mint_amount = Uint256::from(200u128);
        let burn_amount = Uint256::from(100u128);

        // Mint tokens to the owner
        let info = mock_info(lb_pair, &[]);
        let msg = ExecuteMsg::Mint {
            recipient: Addr::unchecked(owner),
            id,
            amount: mint_amount.clone(),
        };
        execute(deps.as_mut(), mock_env(), info, msg)?;

        // Ensure burning from unauthorized sender fails
        let info = mock_info("addr2222", &[]);
        let msg = ExecuteMsg::Burn {
            owner: Addr::unchecked(owner),
            id,
            amount: burn_amount.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(matches!(res.unwrap_err(), Error::Unauthorized));

        // Burn tokens
        let info = mock_info(lb_pair, &[]);
        let msg = ExecuteMsg::Burn {
            owner: Addr::unchecked(owner),
            id,
            amount: burn_amount.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg)?;
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "burn");

        // Check owner's balance
        let owner_canonical = deps.api.addr_canonicalize(owner)?;
        let balance = BALANCES.get(&deps.storage, &(owner_canonical, id)).unwrap();
        assert_eq!(balance, mint_amount - burn_amount);

        // Check total supply
        let total_supply = TOTAL_SUPPLY.get(&deps.storage, &id).unwrap();
        assert_eq!(total_supply, mint_amount - burn_amount);

        Ok(())
    }

    #[test]
    fn test_query_name() -> Result<()> {
        let deps = setup_contract()?;
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Name {})?;
        let value: NameResponse = from_binary(&res)?;
        assert_eq!(value.name, "Token");
        Ok(())
    }

    #[test]
    fn test_query_symbol() -> Result<()> {
        let deps = setup_contract()?;
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Symbol {})?;
        let value: SymbolResponse = from_binary(&res)?;
        assert_eq!(value.symbol, "TKN");
        Ok(())
    }

    #[test]
    fn test_query_decimals() -> Result<()> {
        let deps = setup_contract()?;
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Decimals {})?;
        let value: DecimalsResponse = from_binary(&res)?;
        assert_eq!(value.decimals, 18);
        Ok(())
    }

    #[test]
    fn test_query_total_supply() -> Result<()> {
        let deps = setup_contract()?;
        let id = 1u32;
        let res = query(deps.as_ref(), mock_env(), QueryMsg::TotalSupply { id })?;
        let value: TotalSupplyResponse = from_binary(&res)?;
        assert_eq!(value.total_supply, Uint256::zero());
        Ok(())
    }

    #[test]
    fn test_query_balance_of() -> Result<()> {
        let deps = setup_contract()?;
        let owner = "addr1111";
        let id = 1u32;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::BalanceOf {
                owner: Addr::unchecked(owner),
                id,
            },
        )?;
        let value: BalanceOfResponse = from_binary(&res)?;
        assert_eq!(value.balance, Uint256::zero());
        Ok(())
    }

    #[test]
    fn test_query_balance_of_batch() -> Result<()> {
        let deps = setup_contract()?;
        let owner1 = "addr1111";
        let owner2 = "addr2222";
        let id1 = 1u32;
        let id2 = 2u32;

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::BalanceOfBatch {
                owners: vec![Addr::unchecked(owner1), Addr::unchecked(owner2)],
                ids: vec![id1, id2],
            },
        )
        .unwrap();

        let value: BalanceOfBatchResponse = from_binary(&res).unwrap();
        assert_eq!(value.balances, vec![Uint256::zero(), Uint256::zero()]);
        Ok(())
    }

    #[test]
    fn test_query_is_approved_for_all() -> Result<()> {
        let deps = setup_contract()?;
        let owner = "addr1111";
        let spender = "addr2222";

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsApprovedForAll {
                owner: Addr::unchecked(owner),
                spender: Addr::unchecked(spender),
            },
        )
        .unwrap();

        let value: IsApprovedForAllResponse = from_binary(&res).unwrap();
        assert_eq!(value.is_approved, false);
        Ok(())
    }
}
