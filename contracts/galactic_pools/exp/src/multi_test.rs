#[cfg(test)]
mod tests {

    use std::ops::Add;

    use c_std::{Addr, BlockInfo, Coin, ContractInfo, Empty, StdResult, Uint128};

    use rand::{distributions::Uniform, Rng, SeedableRng};
    use rand_chacha::ChaChaRng;
    use sha2::{Digest, Sha256};
    use shade_protocol::{
        c_std,
        multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor},
    };

    use crate::msg::{AddContract, Entropy, ExecuteMsg};

    const ADMIN: &str = "admin00001";
    const DURATION: u64 = 2400000;
    const MINT_PER_BLOCK: u64 = 1;

    pub fn _pool_info() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    fn mock_app(init_funds: &[Coin]) -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(ADMIN), init_funds.to_vec())
                .unwrap();
        })
    }

    pub fn exp_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }
    #[track_caller]
    fn instantiate_exp(
        app: &mut App,
        address: &Addr,
        rng_contract: Option<ContractInfo>,
    ) -> StdResult<ContractInfo> {
        let flex_id = app.store_code(exp_contract());
        let msg = crate::msg::InstantiateMsg {
            entropy: "entropy lol 123".to_string(),
            admin: Some(vec![address.clone()]),
            schedules: vec![crate::msg::MintingScheduleUint {
                mint_per_block: Uint128::from(MINT_PER_BLOCK),
                duration: DURATION,
                start_after: None,
                continue_with_current_season: false,
            }],

            season_ending_block: 0,
            grand_prize_contract: None,
        };
        Ok(app
            .instantiate_contract(flex_id, Addr::unchecked(ADMIN), &msg, &[], "exp", None)
            .unwrap())
    }

    //1)Init pool contract

    #[test]
    fn test_initialize_exp_contract() -> StdResult<()> {
        let mut app = mock_app(&[]);
        instantiate_exp(&mut app, &Addr::unchecked(ADMIN.to_string()), None)?;
        Ok(())
    }

    #[test]
    fn test_get_winners() -> StdResult<()> {
        //Init rng then exp
        let mut app = mock_app(&[]);
        let exp_contract = instantiate_exp(&mut app, &Addr::unchecked(ADMIN.to_string()), None)?;

        //add contracts
        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract,
                &ExecuteMsg::AddContract {
                    contracts: [AddContract {
                        address: Addr::unchecked("pool1".to_string()),
                        code_hash: "pool1 hash".to_string(),
                        weight: 50,
                    }]
                    .to_vec(),
                },
                &[],
            )
            .unwrap();

        //xp give it to users.
        app.set_block(BlockInfo {
            height: app.block_info().height.add(DURATION),
            time: app.block_info().time,
            chain_id: app.block_info().chain_id,
            random: None,
        });

        let msg = ExecuteMsg::UpdateLastClaimed {};
        let _res = app
            .execute_contract(
                Addr::unchecked(&"pool1".to_string()),
                &exp_contract,
                &msg,
                &[],
            )
            .unwrap();

        //Creating a simulation.

        let total_xp = DURATION * MINT_PER_BLOCK;
        let mut assigned_xp = 0;
        let init_seed_arr = sha2::Sha256::digest("init_seed".as_bytes());
        let init_seed: [u8; 32] = init_seed_arr.as_slice().try_into().expect("Invalid");
        let init_entropy_arr = sha2::Sha256::digest("init_entropy".as_bytes());
        let init_entropy: [u8; 32] = init_entropy_arr.as_slice().try_into().expect("Invalid");
        let entropy = Entropy {
            seed: init_seed,
            entropy: init_entropy,
        };
        while total_xp > assigned_xp {
            //Get a random user here generate a number between 1  and 1000.
            let generate_user_id = generate_random_number(
                &entropy,
                1,
                100000,
                String::from(format!("users {}", assigned_xp)),
            );

            //Get a random xp between 1 and 1000
            let mut generate_xp = generate_random_number(
                &entropy,
                1,
                10000,
                String::from(format!("xp {}", assigned_xp)),
            );

            let random = 10000 % generate_xp;

            generate_xp -= random;

            let msg = ExecuteMsg::AddExp {
                address: Addr::unchecked(format!("user_id {}", generate_user_id).to_string()),
                exp: Uint128::from(generate_xp as u128),
            };

            let _res = app.execute_contract(Addr::unchecked("pool1"), &exp_contract, &msg, &[]);

            assigned_xp += generate_xp;
        }

        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract,
                &ExecuteMsg::SetViewingKey {
                    key: String::from("vk_1"),
                },
                &[],
            )
            .unwrap();

        //get winners

        let query_msg = crate::msg::QueryMsg::GetWinner {
            no_of_winners: Some(1),
            key: String::from("vk_1"),
            address: Addr::unchecked(String::from(ADMIN)),
        };
        let winners_res: crate::msg::QueryAnswer = app
            .wrap()
            .query_wasm_smart(&exp_contract.code_hash, &exp_contract.address, &query_msg)
            .unwrap();

        if let crate::msg::QueryAnswer::GetWinnersResponse { winners } = winners_res {}

        //Start new round

        let msg = ExecuteMsg::ResetSeason {};

        let _res = app
            .execute_contract(
                Addr::unchecked(&ADMIN.to_string()),
                &exp_contract,
                &msg,
                &[],
            )
            .unwrap();
        let query_msg = crate::msg::QueryMsg::GetWinner {
            no_of_winners: Some(1),
            key: String::from("vk_1"),
            address: Addr::unchecked(String::from(ADMIN)),
        };
        let winners_res: Result<crate::msg::QueryAnswer, c_std::StdError> = app
            .wrap()
            .query_wasm_smart(&exp_contract.code_hash, &exp_contract.address, &query_msg);
        //   println!("{:?}", winners_res.unwrap());
        assert!(winners_res.is_err());

        Ok(())
    }

    pub fn generate_random_number(
        entropy: &Entropy,
        low: u64,
        high: u64,
        extra_entropy: String,
    ) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(&entropy.seed);
        hasher.update(&entropy.entropy);
        hasher.update(extra_entropy);

        let seed: [u8; 32] = hasher.finalize().into();

        let rng = ChaChaRng::from_seed(seed);

        let range = Uniform::new_inclusive(low, high);
        let mut digit_generator = rng.clone().sample_iter(&range);
        let drafted_xp = digit_generator.next().unwrap_or(0u64);

        drafted_xp
    }
}
