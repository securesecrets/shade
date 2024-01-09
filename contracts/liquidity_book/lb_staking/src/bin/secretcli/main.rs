mod example_data;

use example_data::*;
use shade_protocol::{
    c_std::{Addr, Binary, ContractInfo, Uint128, Uint256},
    lb_libraries::pair_parameter_helper::PairParameters,
    liquidity_book::{
        lb_pair::RewardsDistribution,
        lb_staking::{
            Auth,
            ExecuteMsg,
            InstantiateMsg,
            InvokeMsg,
            QueryAnswer,
            QueryMsg,
            QueryTxnType,
            QueryWithPermit,
        },
        lb_token::Snip1155ReceiveMsg,
    },
    s_toolkit::permit::Permit,
    snip20::Snip20ReceiveMsg,
    swap::{
        core::{TokenAmount, TokenType},
        router::*,
    },
    utils::asset::RawContract,
    Contract,
};
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::Path,
};

macro_rules! print_instantiate_message {
    ($file:ident, $($var:ident),+ $(,)?) => {
        $(
            writeln!($file,
                "```sh\nsecretcli tx compute instantiate 1 '{}'\n```",
                serde_json::to_string_pretty(&$var).unwrap()
            )?;
            writeln!($file, "")?;
        )+
    };
}

macro_rules! print_execute_messages {
    ($file:ident, $($var:ident),+ $(,)?) => {
        $(
            writeln!($file,
                "### {}\n\n```sh\nsecretcli tx compute execute secret1foobar '{}'\n```",
                stringify!($var),
                serde_json::to_string_pretty(&$var).unwrap()
            )?;
            writeln!($file, "")?;
        )+
    };
}

macro_rules! print_query_messages_with_responses {
      ($file:ident, $(($cmd:ident, $resp:ident)),+ $(,)?) => {
          $(
              writeln!($file,
                  "### {}\n\n```sh\nsecretcli query compute query secret1foobar '{}'\n```\n",
                  stringify!($cmd),
                  serde_json::to_string_pretty(&$cmd).unwrap()
              )?;
              writeln!($file,
                  "#### Response\n\n```json\n{}\n```\n",
                  serde_json::to_string_pretty(&$resp).unwrap()
              )?;
          )+
      };
  }

fn main() -> io::Result<()> {
    let crate_root = &env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let package_name = &env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");
    let file_path = Path::new(crate_root).join(format!("{package_name}.md"));
    let mut file = File::create(file_path)?;

    writeln!(file, "# {package_name}\n")?;

    // -- Instantiate Message
    let instantiate_msg = InstantiateMsg {
        amm_pair: Addr::contract().to_string(),
        lb_token: RawContract::example(),
        admin_auth: RawContract::example(),
        query_auth: RawContract::example(),
        epoch_index: 1,
        epoch_duration: 3600,
        expiry_duration: Some(50),
        recover_funds_receiver: Addr::recipient(),
    };

    writeln!(file, "## Instantiate Message\n")?;
    print_instantiate_message!(file, instantiate_msg);

    writeln!(file, "## Execute Messages\n")?;
    // -- Execute Messages

    //     print_execute_messages!(
    //       file,
    //       swap_tokens_for_exact,
    //       swap_tokens_for_exact_invoke,
    //       register_snip20_token,
    //       recover_funds,
    //       set_config
    //   );

    // -- Execute Messages

    let claim_rewards = ExecuteMsg::ClaimRewards {};
    let end_epoch = ExecuteMsg::EndEpoch {
        rewards_distribution: RewardsDistribution::example(),
    };
    let unstake = ExecuteMsg::Unstake {
        token_ids: vec![1, 2, 3],
        amounts: vec![
            Uint256::from(100u128),
            Uint256::from(200u128),
            Uint256::from(300u128),
        ],
    };
    let snip1155_receive = ExecuteMsg::Snip1155Receive(Snip1155ReceiveMsg::example());
    let receive = ExecuteMsg::Receive(Snip20ReceiveMsg::example());
    let register_reward_tokens = ExecuteMsg::RegisterRewardTokens(vec![ContractInfo::example()]);
    let update_config = ExecuteMsg::UpdateConfig {
        admin_auth: Some(RawContract::example()),
        query_auth: None,
        epoch_duration: Some(100),
        expiry_duration: Some(200),
    };
    let recover_funds = ExecuteMsg::RecoverExpiredFunds {};
    let create_viewing_key = ExecuteMsg::CreateViewingKey {
        entropy: "random_entropy".to_string(),
    };
    let set_viewing_key = ExecuteMsg::SetViewingKey {
        key: "viewing_key".to_string(),
    };
    let revoke_permit = ExecuteMsg::RevokePermit {
        permit_name: "permit_name".to_string(),
    };

    // Responses for ExecuteMsg
    // Assuming that the execute messages do not return a response directly
    // and are handled via contract state or events

    // -- Invoke Messages
    let stake = InvokeMsg::Stake {
        from: Some("from_addr".to_string()),
        padding: None,
    };
    let add_rewards = InvokeMsg::AddRewards {
        start: Some(1234567890u64),
        end: 1234567890u64,
    };

    // Using the macros to print messages and responses
    // Note: Adjust the macros as needed for handling ExecuteMsg and InvokeMsg
    print_execute_messages!(
        file,
        claim_rewards,
        stake,
        unstake,
        snip1155_receive,
        receive,
        end_epoch,
        add_rewards,
        register_reward_tokens,
        update_config,
        recover_funds,
        create_viewing_key,
        set_viewing_key,
        revoke_permit
    );

    // Note: Add a similar macro for InvokeMsg if needed

    // -- Query Messages

    writeln!(file, "## Query Messages with responses\n")?;

    //     print_query_messages_with_responses!(
    //         file,
    //         (swap_simulation_query, swap_simulation_response),
    //         (get_config_query, get_config_response),
    //         (registered_tokens_query, registered_tokens_response)
    //     );

    let contract_info_query = QueryMsg::ContractInfo {};
    let registered_tokens_query = QueryMsg::RegisteredTokens {};
    let id_total_balance_query = QueryMsg::IdTotalBalance {
        id: "token_id".to_string(),
    };
    let balance_query = QueryMsg::Balance {
        token_id: "token_id".to_string(),
        auth: Auth::example(),
    };
    let all_balances_query = QueryMsg::AllBalances {
        auth: Auth::example(),

        page: Some(1),
        page_size: Some(10),
    };
    let liquidity_query = QueryMsg::Liquidity {
        auth: Auth::example(),

        round_index: Some(1234567890u64),
        token_ids: vec![1, 2, 3],
    };
    let transaction_history_query = QueryMsg::TransactionHistory {
        auth: Auth::example(),
        page: Some(1),
        page_size: Some(10),
        txn_type: QueryTxnType::All,
    };

    // Responses for QueryMsg

    let contract_info_response = QueryAnswer::ContractInfo {
        lb_token: ContractInfo::example(),
        lb_pair: Addr::contract(),
        admin_auth: Contract::example(),
        query_auth: Contract::example(),
        epoch_index: 1,
        epoch_durations: 3600,
        expiry_durations: Some(5000), /* ... fill in details ... */
    };
    let registered_tokens_response = QueryAnswer::RegisteredTokens(vec![ContractInfo::example()]);
    let id_total_balance_response = QueryAnswer::IdTotalBalance {
        amount: Uint256::from(123u128),
    };
    let balance_response = QueryAnswer::Balance {
        amount: Uint256::from(123u128),
    };
    let all_balances_response = QueryAnswer::AllBalances(Vec::new()); // Fill with appropriate OwnerBalance structs
    let liquidity_response = QueryAnswer::Liquidity(Vec::new()); // Fill with appropriate Liquidity structs
    let transaction_history_response = QueryAnswer::TransactionHistory {
        txns: Vec::new(), // Fill with appropriate Tx structs
        count: 123,
    };
    //     let viewing_key_error_response = QueryAnswer::ViewingKeyError {
    //         msg: "Error message".to_string(),
    //     };

    // Using the macro to print messages and responses

    print_query_messages_with_responses!(
        file,
        (contract_info_query, contract_info_response),
        (registered_tokens_query, registered_tokens_response),
        (id_total_balance_query, id_total_balance_response),
        (balance_query, balance_response),
        (all_balances_query, all_balances_response),
        (liquidity_query, liquidity_response),
        (transaction_history_query, transaction_history_response),
    );

    Ok(())
}
