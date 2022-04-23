use shade_protocol::governance;
use fadroma_ensemble::MockEnv;
use fadroma_platform_scrt::ContractLink;
use cosmwasm_math_compat::Uint128;
use crate::tests::admin_only_governance;

#[test]
fn trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            metadata: "Proposal metadata".to_string(),
            contract: None,
            assembly_msg: None,
            variables: None,
            padding: None
        },
        MockEnv::new(
            "admin",
            ContractLink {
                address: gov.address,
                code_hash: gov.code_hash,
            }
        )
    ).unwrap();
}

#[test]
fn unauthorized_trigger_admin_command() {
    let (mut chain, gov) = admin_only_governance().unwrap();

    assert!(chain.execute(
        &governance::HandleMsg::AssemblyProposal {
            assembly: Uint128::new(1),
            metadata: "Proposal metadata".to_string(),
            contract: None,
            assembly_msg: None,
            variables: None,
            padding: None
        },
        MockEnv::new(
            "random",
            gov.clone()
        )
    ).is_err());
}

// TODO: Create normal proposal
// TODO: Create text only proposal
// TODO: Create non wasm chain proposal

// TODO: Try assembly voting
// TODO: Try update while in assembly voting
// TODO: Try update on yes
// TODO: Try update on abstain
// TODO: Try update on no
// TODO: Try update on veto

// TODO: try funding
// TODO: Try update while funding
// TODO: Update while fully funded
// TODO: Update after failed funding

// TODO: Try voting
// TODO: Try update while in voting
// TODO: Try update on yes
// TODO: Try update on abstain
// TODO: Try update on no
// TODO: Try update on veto

// TODO: Trigger a failed contract and then cancel
// TODO: Cancel contract