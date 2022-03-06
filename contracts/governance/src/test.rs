#[cfg(test)]
mod tests {
    use crate::contract;
    use cosmwasm_math_compat::Uint128;
    use cosmwasm_std::{
        coins, from_binary,
        testing::{mock_dependencies, mock_env},
        Api, Extern, HumanAddr, Querier, Storage,
    };
    use shade_protocol::utils::asset::Contract;
    use shade_protocol::utils::generic_response::ResponseStatus;
    use shade_protocol::{
        governance,
        governance::proposal::{ProposalStatus, QueriedProposal},
    };

    #[test]
    fn get_proposals_by_status() {
        let mut deps = mock_dependencies(20, &coins(0, ""));

        // Initialize governance contract.
        let env = mock_env("creator", &coins(0, ""));
        let governance_init_msg = governance::InitMsg {
            admin: None,
            // The next governance votes will not require voting
            staker: None,
            funding_token: Contract {
                address: HumanAddr::from(""),
                code_hash: String::from(""),
            },
            funding_amount: Uint128::new(1000000u128),
            funding_deadline: 180,
            voting_deadline: 180,
            // 5 shade is the minimum
            quorum: Uint128::new(5000000u128),
        };
        let res = contract::init(&mut deps, env, governance_init_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // Initialized governance contract has no proposals.
        let res = contract::query(
            &deps,
            governance::QueryMsg::GetProposals {
                start: Uint128::new(0u128),
                end: Uint128::new(100u128),
                status: Some(ProposalStatus::Funding),
            },
        )
        .unwrap();
        let value: governance::QueryAnswer = from_binary(&res).unwrap();
        match value {
            governance::QueryAnswer::Proposals { proposals } => {
                assert_eq!(0, proposals.len());
            }
            _ => {
                panic!("Received wrong answer")
            }
        }

        // Create a proposal on governance contract.
        let env = mock_env("creator", &coins(0, ""));
        let res = contract::handle(
            &mut deps,
            env,
            governance::HandleMsg::CreateProposal {
                target_contract: String::from(governance::GOVERNANCE_SELF),
                proposal: serde_json::to_string(&governance::HandleMsg::AddAdminCommand {
                    name: "random data here".to_string(),
                    proposal: "{\"update_config\":{\"unbond_time\": {}, \"admin\": null}}"
                        .to_string(),
                })
                .unwrap(),
                description: String::from("Proposal on governance contract"),
            },
        )
        .unwrap();
        let value: governance::HandleAnswer = from_binary(&res.data.unwrap()).unwrap();
        match value {
            governance::HandleAnswer::CreateProposal {
                status,
                proposal_id,
            } => {
                assert_eq!(ResponseStatus::Success, status);
                assert!(!proposal_id.is_zero());
            }
            _ => {
                panic!("Received wrong answer")
            }
        }

        // Now we should have single proposal in `funding`.

        // Should return this proposal when no specific status is specified.
        assert_get_proposals(
            &deps,
            governance::QueryMsg::GetProposals {
                start: Uint128::zero(),
                end: Uint128::new(100u128),
                status: None,
            },
            |proposals| {
                assert_eq!(1, proposals.len());
                assert_eq!(proposals[0].status, ProposalStatus::Funding);
            },
        );

        // Should return this proposal when `funding` status is specified.
        assert_get_proposals(
            &deps,
            governance::QueryMsg::GetProposals {
                start: Uint128::zero(),
                end: Uint128::new(100u128),
                status: Some(ProposalStatus::Funding),
            },
            |proposals| {
                assert_eq!(1, proposals.len());
                assert_eq!(proposals[0].status, ProposalStatus::Funding);
            },
        );

        // Shouldn't return this proposal when querying by status different from `funding`.
        assert_get_proposals(
            &deps,
            governance::QueryMsg::GetProposals {
                start: Uint128::zero(),
                end: Uint128::new(100u128),
                status: Some(ProposalStatus::Voting),
            },
            |proposals| {
                assert_eq!(0, proposals.len());
            },
        );
    }

    ///
    /// Assert via assertFn on the result of governance::QueryMsg::GetProposals contract call.
    ///
    /// # Arguments
    ///
    /// * 'deps' - External contract dependencies
    /// * 'msg' - The message data
    /// * 'assert_fn' - A bunch of assert statements to be performed on contract call response
    ///
    pub fn assert_get_proposals<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        msg: governance::QueryMsg,
        assert_fn: fn(result: Vec<QueriedProposal>),
    ) {
        let res = contract::query(&deps, msg).unwrap();
        let value: governance::QueryAnswer = from_binary(&res).unwrap();
        match value {
            governance::QueryAnswer::Proposals { proposals } => assert_fn(proposals),
            _ => {
                panic!("Received wrong answer")
            }
        }
    }
}
