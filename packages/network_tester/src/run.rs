use serde_json::Result;
use secretcli::{cli_types::NetContract, secretcli::{secretcli_run, query_contract}};
use shade_protocol::{mint};
use secretcli::secretcli::TestQuery;

fn main() -> Result<()> {
    let demo = secretcli_run(vec!["query".to_string(),
                                  "compute".to_string(), "list-code".to_string()])?;
    println!("{}", demo[1]);

    // let txhash = store_contract("../../compiled/oracle.wasm.gz",
    //                             Option::from("admin"), None, None)?;
    //
    // println!("{}", txhash.txhash);

    let mint = NetContract {
        label: "mint-GRypoSRJ".to_string(),
        id: "30572".to_string(),
        address: "secret13x46stce2f9s8aukey8nfz9wnfcx6qmdc7c0vy".to_string(),
        code_hash: "F4255F459419F0B9CF1DA23609D69715D2964496A2D918548664AC9F58B196F9".to_string()
    };
    // let query: mint::QueryAnswer = query_contract(
    //     mint, mint::QueryMsg::GetSupportedAssets {})?;

    let query = mint::QueryMsg::GetSupportedAssets {}.t_query(mint)?;

    if let mint::QueryAnswer::SupportedAssets {assets} = query {
        println!("Supported Assets: ");
        for asset in assets {
            println!("\t{},", asset);
        }
    }
    Ok(())
}