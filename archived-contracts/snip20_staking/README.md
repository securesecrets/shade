# SNIP-20 Reference Implementation

This is an implementation of a [SNIP-20](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md), [SNIP-21](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-21.md), [SNIP-22](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-22.md), [SNIP-23](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-23.md) and [SNIP-24](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md) compliant token contract.
At the time of token creation you may configure:
* Public Total Supply:  If you enable this, the token's total supply will be displayed whenever a TokenInfo query is performed.  DEFAULT: false
* Enable Deposit: If you enable this, you will be able to convert from SCRT to the token.*  DEFAULT: false
* Enable Redeem: If you enable this, you will be able to redeem your token for SCRT.*  It should be noted that if you have redeem enabled, but deposit disabled, all redeem attempts will fail unless someone has sent SCRT to the token contract.  DEFAULT: false
* Enable Mint: If you enable this, any address in the list of minters will be able to mint new tokens.  The admin address is the default minter, but can use the set/add/remove_minters functions to change the list of approved minting addresses.  DEFAULT: false
* Enable Burn: If you enable this, addresses will be able to burn tokens.  DEFAULT: false


\*:The conversion rate will be 1 uscrt for 1 minimum denomination of the token.  This means that if your token has 6 decimal places, it will convert 1:1 with SCRT.  If your token has 10 decimal places, it will have an exchange rate of 10000 SCRT for 1 token.  If your token has 3 decimal places, it will have an exchange rate of 1000 tokens for 1 SCRT.  You can use the exchange_rate query to view the exchange rate for the token.  The query response will display either how many tokens are worth 1 SCRT, or how many SCRT are worth 1 token.  That is, the response lists the symbol of the coin that has less value (either SCRT or the token), and the number of those coins that are worth 1 of the other.

## Usage examples:

To create a new token:

```secretcli tx compute instantiate <code-id> '{"name":"<your_token_name>","symbol":"<your_token_symbol>","admin":"<optional_admin_address_defaults_to_the_from_address>","decimals":<number_of_decimals>,"initial_balances":[{"address":"<address1>","amount":"<amount_for_address1>"}],"prng_seed":"<base64_encoded_string>","config":{"public_total_supply":<true_or_false>,"enable_deposit":<true_or_false>,"enable_redeem":<true_or_false>,"enable_mint":<true_or_false>,"enable_burn":<true_or_false>}}' --label <token_label> --from <account>```

The `admin` field is optional and will default to the "--from" address if you do not specify it.  The `initial_balances` field is optional, and you can specify as many addresses/balances as you like.  The `config` field as well as every field in the `config` is optional.  Any `config` fields not specified will default to `false`.

To deposit: ***(This is public)***

```secretcli tx compute execute <contract-address> '{"deposit": {}}' --amount 1000000uscrt --from <account>``` 

To send SSCRT:

```secretcli tx compute execute <contract-address> '{"transfer": {"recipient": "<destination_address>", "amount": "<amount_to_send>"}}' --from <account>```

To set your viewing key: 

```secretcli tx compute execute <contract-address> '{"create_viewing_key": {"entropy": "<random_phrase>"}}' --from <account>```

To check your balance:

```secretcli q compute query <contract-address> '{"balance": {"address":"<your_address>", "key":"your_viewing_key"}}'```

To view your transaction history:

```secretcli q compute query <contract-address> '{"transfer_history": {"address": "<your_address>", "key": "<your_viewing_key>", "page": <optional_page_number>, "page_size": <number_of_transactions_to_return>}}'```

To withdraw: ***(This is public)***

```secretcli tx compute execute <contract-address> '{"redeem": {"amount": "<amount_in_smallest_denom_of_token>"}}' --from <account>```

To view the token contract's configuration:

```secretcli q compute query <contract-address> '{"token_config": {}}'```

To view the deposit/redeem exchange rate:

```secretcli q compute query <contract-address> '{"exchange_rate": {}}'```


## Troubleshooting 

All transactions are encrypted, so if you want to see the error returned by a failed transaction, you need to use the command

`secretcli q compute tx <TX_HASH>`
