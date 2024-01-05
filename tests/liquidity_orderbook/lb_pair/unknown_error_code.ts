// function liquidityParametersGenerator(
//       binStep: number,
//       activeId: number,
//       tokenX: TokenType,
//       tokenY: TokenType,
//       amountX: number, // Assuming Uint128 is represented as bigint in TS
//       amountY: number,
//       nbBinsX: number,
//       nbBinsY: number
//     ): LiquidityParameters {
//       if (activeId > 2 ** 24 - 1) {
//         throw new Error("active_id too big");
//       }

//       console.log("active_id: " + activeId);

//       let total = get_total_bins(nbBinsX, nbBinsY); // Implement get_total_bins function
//       activeId = activeId + total;

//       let distributionX: number[] = [];
//       let distributionY: number[] = [];
//       let deltaIds: number[] = [];

//       for (let i = 0; i < total; i++) {
//         if (nbBinsY > 0) {
//           deltaIds.push(i - nbBinsY + 1);
//         } else {
//           deltaIds.push(i);
//         }

//         let id = get_id(activeId, i, nbBinsY); // Implement get_id function
//         let distribX =
//           id >= activeId && nbBinsX > 0 ? safe64Divide(PRECISION, nbBinsX) : 0;
//         let distribY =
//           id <= activeId && nbBinsY > 0 ? safe64Divide(PRECISION, nbBinsY) : 0;

//         distributionX.push(distribX);
//         distributionY.push(distribY);
//       }

//       let liquidityParameters: LiquidityParameters = {
//         active_id_desired: activeId,
//         amount_x: `${amountX}`,
//         amount_x_min: `${multiplyRatio(amountX, 90, 100)}`,
//         amount_y: `${amountY}`,
//         amount_y_min: `${multiplyRatio(amountY, 90, 100)}`,
//         bin_step: binStep,
//         deadline: 999999999999,
//         delta_ids: deltaIds,
//         distribution_x: distributionX,
//         distribution_y: distributionY,
//         id_slippage: 15,
//         token_x: tokenX,
//         token_y: tokenY,
//       };

//       return liquidityParameters;
//     }

//     // Implement safe64Divide, get_total_bins, get_id, multiplyRatio and any other required functions or logic

//     export async function executeAddLiquidity(
//       client: SecretNetworkClient,
//       contractHashPair: string,
//       contractAddressPair: string,
//       bin_step: number,
//       tokenX: LBPair.TokenType,
//       tokenY: LBPair.TokenType,
//       no_of_bins_x: number,
//       no_of_bins_y: number
//     ) {
//       let liquidity_params = liquidityParametersGenerator(
//         bin_step,
//         2 ** 23,
//         tokenX,
//         tokenY,
//         340282366920938463340,
//         340282366920938463345,
//         no_of_bins_x,
//         no_of_bins_y
//       );
//       // ("340282366920938463463374607431768211454");

//       const msg: LBPair.ExecuteMsg = {
//         add_liquidity: {
//           liquidity_parameters: liquidity_params,
//         },
//       };

//       const tx = await client.tx.compute.executeContract(
//         {
//           sender: client.address,
//           contract_address: contractAddressPair,
//           code_hash: contractHashPair,
//           msg: msg,
//           sent_funds: [],
//         },
//         {
//           gasLimit: 6000000,
//         }
//       );
//       if ("custom_token" in tokenX && "custom_token" in tokenY) {
//         let contractX = {
//           address: tokenX.custom_token.contract_addr,
//           code_hash: tokenX.custom_token.token_code_hash,
//         };
//         let balanceX = await client.query.snip20.getBalance({
//           contract: contractX,
//           address: client.address,
//           auth: {
//             key: "viewing_key",
//           },
//         });

//         console.log(balanceX);
//       }
//       if (tx.code !== 0) {
//         throw new Error(`Failed with the following error:\n ${tx.rawLog}`);
//       }

//       let total_bins = get_total_bins(no_of_bins_x, no_of_bins_y);

//       console.log(
//         `Add Liquidity TX used ${tx.gasUsed} gas, total_bins ${total_bins}`
//       );
//     }
