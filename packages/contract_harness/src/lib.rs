use cosmwasm_std::{from_binary, Binary, Env, HandleResponse, InitResponse, StdResult};
use fadroma::ensemble::{ContractHarness, MockDeps};

macro_rules! implement_harness {
    ($x:ident, $s:ident) => {
        impl ContractHarness for $x {
            fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
                $s::contract::init(deps, env, from_binary(&msg)?)
            }

            fn handle(
                &self,
                deps: &mut MockDeps,
                env: Env,
                msg: Binary,
            ) -> StdResult<HandleResponse> {
                $s::contract::handle(deps, env, from_binary(&msg)?)
            }

            fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
                $s::contract::query(deps, from_binary(&msg)?)
            }
        }
    };
}

use snip20;
pub struct Snip20;
implement_harness!(Snip20, snip20);

use snip20_reference_impl;
pub struct Snip20ReferenceImpl;
implement_harness!(Snip20ReferenceImpl, snip20_reference_impl);

use mint;
pub struct Mint;
implement_harness!(Mint, mint);

use oracle;
pub struct Oracle;
implement_harness!(Oracle, oracle);

use mock_band;
pub struct MockBand;
implement_harness!(MockBand, mock_band);

use treasury;
pub struct Treasury;
implement_harness!(Treasury, treasury);

use treasury_manager;
pub struct TreasuryManager;
implement_harness!(TreasuryManager, treasury_manager);

use scrt_staking;
pub struct ScrtStaking;
implement_harness!(ScrtStaking, scrt_staking);

use governance;
pub struct Governance;
implement_harness!(Governance, governance);

use spip_stkd_0;
pub struct Snip20Staking;
implement_harness!(Snip20Staking, spip_stkd_0);

use mock_sienna_pair;
pub struct MockSiennaPair;
implement_harness!(MockSiennaPair, mock_sienna_pair);
