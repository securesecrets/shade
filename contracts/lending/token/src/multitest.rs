pub mod controller;
// pub mod rebasing;
pub mod receiver;
pub mod suite;

use cosmwasm_std::{coins, Decimal, Event, Uint128};
use utils::coin::coin_native;

use crate::msg::TokenInfoResponse;
use crate::ContractError;
use suite::{Suite, SuiteBuilder};

#[test]
fn fresh_queries() {
    let suite = SuiteBuilder::new()
        .with_name("WyndLend")
        .with_symbol("LDX")
        .with_decimals(9)
        .build();
    let actor = "actor";
    let controller = suite.controller();
    let controller = controller.as_str();

    assert_eq!(
        suite.query_token_info().unwrap(),
        TokenInfoResponse {
            name: "WyndLend".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            total_supply: Uint128::zero(),
            multiplier: Decimal::from_ratio(1u128, 100_000u128)
        }
    );

    assert_eq!(suite.query_balance(actor).unwrap(), Uint128::zero());
    assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    // assert_eq!(
    //     suite.query_distributed_funds().unwrap(),
    //     coin_native(0, "Reward")
    // );
    // assert_eq!(
    //     suite.query_undistributed_funds().unwrap(),
    //     coin_native(0, "Reward")
    // );
}

// mod minting {
//     use super::*;
//
//     #[test]
//     fn by_controller() {
//         let mut suite = Suite::new();
//         let lender = "lender";
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn zero_amount() {
//         let mut suite = Suite::new();
//         let lender = "lender";
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         let err = suite.mint(controller, lender, Uint128::zero()).unwrap_err();
//
//         assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn by_non_controller() {
//         let mut suite = Suite::new();
//         let lender = "lender";
//         let minter = "minter";
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         let err = suite.mint(minter, lender, Uint128::new(100)).unwrap_err();
//
//         assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(minter).unwrap(), Uint128::zero());
//     }
// }
//
// mod burning {
//     use super::*;
//
//     #[test]
//     fn by_controller() {
//         let mut suite = Suite::new();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let holder = "holder";
//
//         // Preparation to have anything to burn
//         suite.mint(controller, holder, Uint128::new(100)).unwrap();
//
//         // Actually burning
//         suite.burn(controller, holder, Uint128::new(50)).unwrap();
//
//         assert_eq!(suite.query_balance(holder).unwrap(), Uint128::new(50u128));
//     }
//
//     #[test]
//     fn zero_amount() {
//         let mut suite = Suite::new();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let holder = "holder";
//
//         // Preparation to have anything to burn
//         suite.mint(controller, holder, Uint128::new(100)).unwrap();
//
//         // Actually burning
//         let err = suite.burn(controller, holder, Uint128::zero()).unwrap_err();
//
//         assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
//         assert_eq!(suite.query_balance(holder).unwrap(), Uint128::new(100u128));
//     }
//
//     #[test]
//     fn overflow() {
//         let mut suite = Suite::new();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let holder = "holder";
//
//         // Preparation to have anything to burn
//         suite.mint(controller, holder, Uint128::new(100)).unwrap();
//
//         // Actually burning
//         let err = suite
//             .burn(controller, holder, Uint128::new(150))
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::InsufficientTokens {
//                 available: Uint128::new(100),
//                 needed: Uint128::new(150u128)
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_balance(holder).unwrap(), Uint128::new(100u128));
//     }
//
//     #[test]
//     fn by_non_controller() {
//         let mut suite = Suite::new();
//         let lender = "lender";
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         // Preparation to have anything to burn
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         // Actually burning
//         let err = suite.burn(lender, lender, Uint128::new(150)).unwrap_err();
//
//         assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
// }
//
// mod transfer {
//     use super::*;
//
//     #[test]
//     fn proper() {
//         let lender = "lender";
//         let receiver = "receiver";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         suite.transfer(lender, receiver, Uint128::new(40)).unwrap();
//
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(60u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::new(40u128));
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn zero_amount() {
//         let lender = "lender";
//         let receiver = "receiver";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         let err = suite
//             .transfer(lender, receiver, Uint128::zero())
//             .unwrap_err();
//
//         assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn overflow() {
//         let lender = "lender";
//         let receiver = "receiver";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(200))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         let err = suite
//             .transfer(lender, receiver, Uint128::new(140))
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::InsufficientTokens {
//                 available: Uint128::new(100),
//                 needed: Uint128::new(140)
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn not_enough_transferable() {
//         let lender = "lender";
//         let receiver = "receiver";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(200)).unwrap();
//
//         let err = suite
//             .transfer(lender, receiver, Uint128::new(140))
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::CannotTransfer {
//                 max_transferable: Uint128::new(100),
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(200u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn no_transferable() {
//         let lender = "lender";
//         let receiver = "receiver";
//         let mut suite = Suite::new();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(200)).unwrap();
//
//         let err = suite
//             .transfer(lender, receiver, Uint128::new(140))
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::CannotTransfer {
//                 max_transferable: Uint128::zero(),
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(200u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
// }
//
// mod send {
//     use super::*;
//     use crate::multitest::receiver::Cw20ExecMsg;
//     use cosmwasm_std::to_binary;
//
//     #[test]
//     fn proper() {
//         let lender = "lender";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let receiver = suite.receiver();
//         let receiver = receiver.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();
//
//         suite
//             .send(lender, receiver, Uint128::new(40), exec)
//             .unwrap();
//
//         assert_eq!(suite.query_receiver().unwrap(), 1);
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(60u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::new(40u128));
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn zero_amount() {
//         let lender = "lender";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let receiver = suite.receiver();
//         let receiver = receiver.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();
//
//         let err = suite
//             .send(lender, receiver, Uint128::zero(), exec)
//             .unwrap_err();
//
//         assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
//         assert_eq!(suite.query_receiver().unwrap(), 0);
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn remote_call_fails() {
//         let lender = "lender";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let receiver = suite.receiver();
//         let receiver = receiver.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         let exec = to_binary(&Cw20ExecMsg::Invalid {}).unwrap();
//
//         let err = suite
//             .send(lender, receiver, Uint128::new(40), exec)
//             .unwrap_err();
//
//         assert!(err.to_string().contains("error executing WasmMsg"));
//         assert_eq!(suite.query_receiver().unwrap(), 0);
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn overflow() {
//         let lender = "lender";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(200))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let receiver = suite.receiver();
//         let receiver = receiver.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(100)).unwrap();
//
//         let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();
//
//         let err = suite
//             .send(lender, receiver, Uint128::new(140), exec)
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::InsufficientTokens {
//                 available: Uint128::new(100),
//                 needed: Uint128::new(140)
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_receiver().unwrap(), 0);
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn not_enough_transferable() {
//         let lender = "lender";
//         let mut suite = SuiteBuilder::new()
//             .with_transferable(lender, Uint128::new(100))
//             .build();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let receiver = suite.receiver();
//         let receiver = receiver.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(200)).unwrap();
//
//         let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();
//
//         let err = suite
//             .send(lender, receiver, Uint128::new(140), exec)
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::CannotTransfer {
//                 max_transferable: Uint128::new(100),
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_receiver().unwrap(), 0);
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(200u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
//
//     #[test]
//     fn no_transferable() {
//         let lender = "lender";
//         let mut suite = Suite::new();
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let receiver = suite.receiver();
//         let receiver = receiver.as_str();
//
//         // Preparation to have anything to transfer
//         suite.mint(controller, lender, Uint128::new(200)).unwrap();
//
//         let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();
//
//         let err = suite
//             .send(lender, receiver, Uint128::new(140), exec)
//             .unwrap_err();
//
//         assert_eq!(
//             ContractError::CannotTransfer {
//                 max_transferable: Uint128::zero(),
//             },
//             err.downcast().unwrap()
//         );
//         assert_eq!(suite.query_receiver().unwrap(), 0);
//         assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(200u128));
//         assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::zero());
//         assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
//     }
// }
//
// mod distribution {
//     use utils::coin::coin_native;
//
//     use super::*;
//
//     fn distribution_event(sender: &str, denom: &str, amount: u128) -> Event {
//         Event::new("wasm")
//             .add_attribute("sender", sender)
//             .add_attribute("denom", denom)
//             .add_attribute("amount", amount.to_string())
//     }
//
//     #[test]
//     fn divisible_amount_distributed() {
//         let members = ["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(400, reward))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(1)).unwrap();
//         suite.mint(controller, members[1], Uint128::new(2)).unwrap();
//         suite.mint(controller, members[2], Uint128::new(5)).unwrap();
//
//         // Funds distribution
//         let resp = suite
//             .distribute(members[3], None, &coins(400, reward))
//             .unwrap();
//
//         resp.assert_event(&distribution_event(members[3], reward, 400));
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 400);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 0);
//
//         assert_eq!(
//             suite.query_withdrawable_funds(members[0]).unwrap(),
//             coin_native(50, reward)
//         );
//         assert_eq!(
//             suite.query_withdrawable_funds(members[1]).unwrap(),
//             coin_native(100, reward)
//         );
//         assert_eq!(
//             suite.query_withdrawable_funds(members[2]).unwrap(),
//             coin_native(250, reward)
//         );
//
//         assert_eq!(
//             suite.query_distributed_funds().unwrap(),
//             coin_native(400, reward)
//         );
//         assert_eq!(
//             suite.query_undistributed_funds().unwrap(),
//             coin_native(0, reward)
//         );
//
//         // Funds withdrawal
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 50);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 250);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 0);
//     }
//
//     #[test]
//     fn divisible_amount_distributed_twice() {
//         let members = ["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(1000, reward))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(1)).unwrap();
//         suite.mint(controller, members[1], Uint128::new(2)).unwrap();
//         suite.mint(controller, members[2], Uint128::new(5)).unwrap();
//
//         suite
//             .distribute(members[3], None, &coins(400, reward))
//             .unwrap();
//
//         assert_eq!(
//             suite.query_distributed_funds().unwrap(),
//             coin_native(400, reward)
//         );
//         assert_eq!(
//             suite.query_undistributed_funds().unwrap(),
//             coin_native(0, reward)
//         );
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 50);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 250);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 600);
//
//         suite
//             .distribute(members[3], None, &coins(600, reward))
//             .unwrap();
//
//         assert_eq!(
//             suite.query_distributed_funds().unwrap(),
//             coin_native(1000, reward)
//         );
//         assert_eq!(
//             suite.query_undistributed_funds().unwrap(),
//             coin_native(0, reward)
//         );
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 125);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 250);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 625);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 0);
//     }
//
//     #[test]
//     fn divisible_amount_distributed_twice_accumulated() {
//         let members = ["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(1000, reward))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(1)).unwrap();
//         suite.mint(controller, members[1], Uint128::new(2)).unwrap();
//         suite.mint(controller, members[2], Uint128::new(5)).unwrap();
//
//         suite
//             .distribute(members[3], None, &coins(400, reward))
//             .unwrap();
//
//         assert_eq!(
//             suite.query_distributed_funds().unwrap(),
//             coin_native(400, reward)
//         );
//         assert_eq!(
//             suite.query_undistributed_funds().unwrap(),
//             coin_native(0, reward)
//         );
//
//         suite
//             .distribute(members[3], None, &coins(600, reward))
//             .unwrap();
//
//         assert_eq!(
//             suite.query_distributed_funds().unwrap(),
//             coin_native(1000, reward)
//         );
//         assert_eq!(
//             suite.query_undistributed_funds().unwrap(),
//             coin_native(0, reward)
//         );
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 125);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 250);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 625);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 0);
//     }
//
//     #[test]
//     fn weight_changed_after_distribution() {
//         let members = vec!["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(1500, reward))
//             .with_transferable(members[1], Uint128::new(1))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(1)).unwrap();
//         suite.mint(controller, members[1], Uint128::new(2)).unwrap();
//         suite.mint(controller, members[2], Uint128::new(5)).unwrap();
//
//         // First distribution
//         suite
//             .distribute(members[3], None, &coins(400, reward))
//             .unwrap();
//
//         // Modifying weights to:
//         // member[0] => 6
//         // member[1] => 0 (removed)
//         // member[2] => 5
//         // total_weight => 11
//         suite
//             .transfer(members[1], members[0], Uint128::new(1))
//             .unwrap();
//         suite.mint(controller, members[0], Uint128::new(4)).unwrap();
//         suite.burn(controller, members[1], Uint128::new(1)).unwrap();
//
//         // Ensure funds are withdrawn properly, considering old weights
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 50);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 250);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 1100);
//
//         // Distribute tokens again to ensure distribution considers new weights
//         suite
//             .distribute(members[3], None, &coins(1100, reward))
//             .unwrap();
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 650);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 750);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 0);
//     }
//
//     #[test]
//     fn weight_changed_after_distribution_accumulated() {
//         let members = vec!["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(1500, reward))
//             .with_transferable(members[1], Uint128::new(1))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(1)).unwrap();
//         suite.mint(controller, members[1], Uint128::new(2)).unwrap();
//         suite.mint(controller, members[2], Uint128::new(5)).unwrap();
//
//         // First distribution
//         suite
//             .distribute(members[3], None, &coins(400, reward))
//             .unwrap();
//
//         // Modifying weights to:
//         // member[0] => 6
//         // member[1] => 0 (removed)
//         // member[2] => 5
//         // total_weight => 11
//         suite
//             .transfer(members[1], members[0], Uint128::new(1))
//             .unwrap();
//         suite.mint(controller, members[0], Uint128::new(4)).unwrap();
//         suite.burn(controller, members[1], Uint128::new(1)).unwrap();
//
//         // Distribute tokens again to ensure distribution considers new weights
//         suite
//             .distribute(members[3], None, &coins(1100, reward))
//             .unwrap();
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 650);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 750);
//         assert_eq!(suite.native_balance(members[3], reward).unwrap(), 0);
//     }
//
//     #[test]
//     fn distribution_with_leftover() {
//         let members = ["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(3100, reward))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(7)).unwrap();
//         suite
//             .mint(controller, members[1], Uint128::new(11))
//             .unwrap();
//         suite
//             .mint(controller, members[2], Uint128::new(13))
//             .unwrap();
//
//         suite
//             .distribute(members[3], None, &coins(100, reward))
//             .unwrap();
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 2);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 22);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 35);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 41);
//
//         suite
//             .distribute(members[3], None, &coins(3000, reward))
//             .unwrap();
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 700);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 1100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 1300);
//     }
//
//     #[test]
//     fn distribution_with_leftover_accumulated() {
//         let members = ["member1", "member2", "member3", "member4"];
//         let reward = "Reward";
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_native_token(reward)
//             .with_funds(members[3], coins(3100, reward))
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//
//         // Mint tokens to have something to base on
//         suite.mint(controller, members[0], Uint128::new(7)).unwrap();
//         suite
//             .mint(controller, members[1], Uint128::new(11))
//             .unwrap();
//         suite
//             .mint(controller, members[2], Uint128::new(13))
//             .unwrap();
//
//         suite
//             .distribute(members[3], None, &coins(100, reward))
//             .unwrap();
//
//         suite
//             .distribute(members[3], None, &coins(3000, reward))
//             .unwrap();
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//         suite.withdraw_funds(members[2]).unwrap();
//
//         assert_eq!(suite.native_balance(token, reward).unwrap(), 0);
//         assert_eq!(suite.native_balance(members[0], reward).unwrap(), 700);
//         assert_eq!(suite.native_balance(members[1], reward).unwrap(), 1100);
//         assert_eq!(suite.native_balance(members[2], reward).unwrap(), 1300);
//     }
//
//     fn cw20_coin(address: &str, amount: u128) -> cw20::Cw20Coin {
//         cw20::Cw20Coin {
//             address: address.to_string(),
//             amount: amount.into(),
//         }
//     }
//
//     #[test]
//     fn cw20_distributed() {
//         let members = ["member1", "member2"];
//
//         let mut suite = SuiteBuilder::new()
//             .with_distributed_cw20_token(2, vec![cw20_coin(members[0], 1500)])
//             .build();
//
//         let controller = suite.controller();
//         let controller = controller.as_str();
//         let token = suite.token();
//         let token = token.as_str();
//         let dist_token = suite.distributed_token().cw20().unwrap();
//
//         // Mint tokens to have weights for distribution. member1 will get 1/3 and member2 2/3
//         suite.mint(controller, members[0], Uint128::new(5)).unwrap();
//         suite
//             .mint(controller, members[1], Uint128::new(10))
//             .unwrap();
//
//         suite
//             .cw20_send_to_token_contract(&dist_token, members[0], 1500)
//             .unwrap();
//
//         suite.distribute(members[0], None, &[]).unwrap();
//
//         suite.withdraw_funds(members[0]).unwrap();
//         suite.withdraw_funds(members[1]).unwrap();
//
//         assert_eq!(suite.cw20_balance(&dist_token, token).unwrap(), 0);
//         assert_eq!(suite.cw20_balance(&dist_token, members[0]).unwrap(), 500);
//         assert_eq!(suite.cw20_balance(&dist_token, members[1]).unwrap(), 1000);
//     }
// }
