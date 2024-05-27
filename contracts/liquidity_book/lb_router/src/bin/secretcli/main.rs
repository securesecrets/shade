mod example_data;

use example_data::*;
use lb_libraries::pair_parameter_helper::PairParameters;
use shade_protocol::{
    c_std::{Addr, Binary, ContractInfo, Uint128},
    liquidity_book::lb_pair::{InvokeMsg, SwapResult},
    swap::{
        core::{TokenAmount, TokenType},
        router::*,
    },
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
    let mut file = File::create(file_path.clone())?;

    writeln!(file, "# {package_name}\n")?;

    // -- Instantiate Message

    let mut preset = PairParameters::default();

    preset
        .set_static_fee_parameters(
            DEFAULT_BASE_FACTOR,
            DEFAULT_FILTER_PERIOD,
            DEFAULT_DECAY_PERIOD,
            DEFAULT_REDUCTION_FACTOR,
            DEFAULT_VARIABLE_FEE_CONTROL,
            DEFAULT_PROTOCOL_SHARE,
            DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        )
        .unwrap();

    let instantiate_msg = InitMsg {
        prng_seed: Binary::from("prng_seed".as_bytes()),
        entropy: Binary::from("entropy".as_bytes()),
        admin_auth: Contract::example(),
        airdrop_address: None,
    };

    writeln!(file, "## Instantiate Message\n")?;
    print_instantiate_message!(file, instantiate_msg);

    writeln!(file, "## Execute Messages\n")?;
    // -- Execute Messages

    let swap_tokens_for_exact = ExecuteMsg::SwapTokensForExact {
        offer: TokenAmount::example(),
        expected_return: Some(Uint128::from(123u128)),
        path: vec![Hop {
            addr: ContractInfo::example().address.to_string(),
            code_hash: ContractInfo::example().code_hash,
        }],
        recipient: Some("recipient_addr".to_string()),
        padding: None,
    };

    let register_snip20_token = ExecuteMsg::RegisterSNIP20Token {
        token_addr: "token_addr".to_string(),
        token_code_hash: "code_hash".to_string(),
        oracle_key: Some("oracle_key".to_string()),
        padding: None,
    };

    let recover_funds = ExecuteMsg::RecoverFunds {
        token: TokenType::example(),
        amount: Uint128::from(1000u128),
        to: "recipient_addr".to_string(),
        msg: None,
        padding: None,
    };

    let set_config = ExecuteMsg::SetConfig {
        admin_auth: Some(Contract::example()),
        padding: None,
    };

    // Responses for ExecuteMsg
    // Note: Assuming that the execute messages do not return a response directly
    // and are handled via contract state or events

    // -- Invoke Messages

    let swap_tokens_for_exact_invoke = InvokeMsg::SwapTokens {
        expected_return: Some(Uint128::from(123u128)),
        to: None,
        padding: None,
    };

    // Responses for InvokeMsg
    // Note: Assuming that the invoke messages do not return a response directly
    // and are handled via contract state or events

    // -- Query Messages
    writeln!(file, "## Query Messages with responses\n")?;

    let swap_simulation_query = QueryMsg::SwapSimulation {
        offer: TokenAmount::example(),
        path: vec![Hop {
            addr: ContractInfo::example().address.to_string(),
            code_hash: ContractInfo::example().code_hash,
        }],
        exclude_fee: Some(true),
    };

    let get_config_query = QueryMsg::GetConfig {};
    let registered_tokens_query = QueryMsg::RegisteredTokens {};

    // Responses for QueryMsg

    let swap_simulation_response = QueryMsgResponse::SwapSimulation {
        total_fee_amount: Uint128::from(100u128),
        lp_fee_amount: Uint128::from(10u128),
        shade_dao_fee_amount: Uint128::from(5u128),
        result: SwapResult {
            return_amount: Uint128::from(100_000u128),
        },
        price: "123.45".to_string(),
    };

    let get_config_response = QueryMsgResponse::GetConfig {
        admin_auth: Contract::example(),
        airdrop_address: Some(Contract::example()),
    };

    let registered_tokens_response = QueryMsgResponse::RegisteredTokens {
        tokens: vec![
            Addr::unchecked("token_addr1"),
            Addr::unchecked("token_addr2"),
        ],
    };

    // Using the macros to print messages and responses
    // Note: Adjust the macros as needed for handling ExecuteMsg and InvokeMsg
    print_execute_messages!(
        file,
        swap_tokens_for_exact,
        swap_tokens_for_exact_invoke,
        register_snip20_token,
        recover_funds,
        set_config
    );

    // Note: Add a similar macro for InvokeMsg if needed

    print_query_messages_with_responses!(
        file,
        (swap_simulation_query, swap_simulation_response),
        (get_config_query, get_config_response),
        (registered_tokens_query, registered_tokens_response)
    );

    println!("Created {}", file_path.display());

    Ok(())
}
