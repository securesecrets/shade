# router

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "prng_seed": "cHJuZ19zZWVk",
  "entropy": "ZW50cm9weQ==",
  "admin_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "airdrop_address": null
}'
```

## Execute Messages

## Query Messages with responses

### swap_tokens_for_exact

```sh
secretcli tx compute execute secret1foobar '{
  "swap_tokens_for_exact": {
    "offer": {
      "token": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "amount": "100"
    },
    "expected_return": "123",
    "path": [
      {
        "addr": "secret1...foobar",
        "code_hash": "0123456789ABCDEF"
      }
    ],
    "recipient": "recipient_addr",
    "padding": null
  }
}'
```

### swap_tokens_for_exact_invoke

```sh
secretcli tx compute execute secret1foobar '{
  "swap_tokens": {
    "expected_return": "123",
    "to": null,
    "padding": null
  }
}'
```

### register_snip20_token

```sh
secretcli tx compute execute secret1foobar '{
  "register_s_n_i_p20_token": {
    "token_addr": "token_addr",
    "token_code_hash": "code_hash",
    "oracle_key": "oracle_key",
    "padding": null
  }
}'
```

### recover_funds

```sh
secretcli tx compute execute secret1foobar '{
  "recover_funds": {
    "token": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    },
    "amount": "1000",
    "to": "recipient_addr",
    "msg": null,
    "padding": null
  }
}'
```

### set_config

```sh
secretcli tx compute execute secret1foobar '{
  "set_config": {
    "admin_auth": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "padding": null
  }
}'
```

### swap_simulation_query

```sh
secretcli query compute query secret1foobar '{
  "swap_simulation": {
    "offer": {
      "token": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "amount": "100"
    },
    "path": [
      {
        "addr": "secret1...foobar",
        "code_hash": "0123456789ABCDEF"
      }
    ],
    "exclude_fee": true
  }
}'
```

#### Response

```json
{
  "swap_simulation": {
    "total_fee_amount": "100",
    "lp_fee_amount": "10",
    "shade_dao_fee_amount": "5",
    "result": {
      "return_amount": "100000"
    },
    "price": "123.45"
  }
}
```

### get_config_query

```sh
secretcli query compute query secret1foobar '{
  "get_config": {}
}'
```

#### Response

```json
{
  "get_config": {
    "admin_auth": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "airdrop_address": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    }
  }
}
```

### registered_tokens_query

```sh
secretcli query compute query secret1foobar '{
  "registered_tokens": {}
}'
```

#### Response

```json
{
  "registered_tokens": {
    "tokens": [
      "token_addr1",
      "token_addr2"
    ]
  }
}
```

