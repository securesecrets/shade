# lb-staking

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "amm_pair": "secret1...foobar",
  "lb_token": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "admin_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "query_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "epoch_index": 1,
  "epoch_duration": 3600,
  "expiry_duration": 50,
  "recover_funds_receiver": "secret1...recipient"
}'
```

## Execute Messages

### claim_rewards

```sh
secretcli tx compute execute secret1foobar '{
  "claim_rewards": {}
}'
```

### stake

```sh
secretcli tx compute execute secret1foobar '{
  "stake": {
    "from": "from_addr",
    "padding": null
  }
}'
```

### unstake

```sh
secretcli tx compute execute secret1foobar '{
  "unstake": {
    "token_ids": [
      1,
      2,
      3
    ],
    "amounts": [
      "100",
      "200",
      "300"
    ]
  }
}'
```

### snip1155_receive

```sh
secretcli tx compute execute secret1foobar '{
  "snip1155_receive": {
    "sender": "secret1...sender",
    "token_id": "8388608",
    "from": "secret1...sender",
    "amount": "0",
    "msg": null
  }
}'
```

### receive

```sh
secretcli tx compute execute secret1foobar '{
  "receive": {
    "sender": "secret1...foobar",
    "from": "secret1...sender",
    "amount": "100",
    "msg": "ImJhc2U2NCBlbmNvZGVkIHN0cmluZyI="
  }
}'
```

### end_epoch

```sh
secretcli tx compute execute secret1foobar '{
  "end_epoch": {
    "rewards_distribution": {
      "ids": [
        8388604,
        8388605,
        8388606,
        8388607,
        8388608,
        8388609,
        8388610,
        8388611,
        8388612,
        8388613
      ],
      "weightages": [
        1000,
        1000,
        1000,
        1000,
        1000,
        1000,
        1000,
        1000,
        1000,
        1000
      ],
      "denominator": 10000
    },
    "epoch_index": 0
  }
}'
```

### add_rewards

```sh
secretcli tx compute execute secret1foobar '{
  "add_rewards": {
    "start": 1234567890,
    "end": 1234567890
  }
}'
```

### register_reward_tokens

```sh
secretcli tx compute execute secret1foobar '{
  "register_reward_tokens": [
    {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    }
  ]
}'
```

### update_config

```sh
secretcli tx compute execute secret1foobar '{
  "update_config": {
    "admin_auth": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "query_auth": null,
    "epoch_duration": 100,
    "expiry_duration": 200
  }
}'
```

### recover_funds

```sh
secretcli tx compute execute secret1foobar '{
  "recover_expired_funds": {}
}'
```

## Query Messages with responses

### contract_info_query

```sh
secretcli query compute query secret1foobar '{
  "contract_info": {}
}'
```

#### Response

```json
{
  "contract_info": {
    "lb_token": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "lb_pair": "secret1...foobar",
    "admin_auth": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "query_auth": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "epoch_index": 1,
    "epoch_durations": 3600,
    "expiry_durations": 5000
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
  "registered_tokens": [
    {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    }
  ]
}
```

### id_total_balance_query

```sh
secretcli query compute query secret1foobar '{
  "id_total_balance": {
    "id": "token_id"
  }
}'
```

#### Response

```json
{
  "id_total_balance": {
    "amount": "123"
  }
}
```

### balance_query

```sh
secretcli query compute query secret1foobar '{
  "balance": {
    "auth": {
      "viewing_key": {
        "key": "viewing_key",
        "address": "secret1...staker"
      }
    },
    "token_id": "token_id"
  }
}'
```

#### Response

```json
{
  "balance": {
    "amount": "123"
  }
}
```

### all_balances_query

```sh
secretcli query compute query secret1foobar '{
  "all_balances": {
    "auth": {
      "viewing_key": {
        "key": "viewing_key",
        "address": "secret1...staker"
      }
    },
    "page": 1,
    "page_size": 10
  }
}'
```

#### Response

```json
{
  "all_balances": []
}
```

### liquidity_query

```sh
secretcli query compute query secret1foobar '{
  "liquidity": {
    "auth": {
      "viewing_key": {
        "key": "viewing_key",
        "address": "secret1...staker"
      }
    },
    "round_index": 1234567890,
    "token_ids": [
      1,
      2,
      3
    ]
  }
}'
```

#### Response

```json
{
  "liquidity": []
}
```

### transaction_history_query

```sh
secretcli query compute query secret1foobar '{
  "transaction_history": {
    "auth": {
      "viewing_key": {
        "key": "viewing_key",
        "address": "secret1...staker"
      }
    },
    "page": 1,
    "page_size": 10,
    "txn_type": "all"
  }
}'
```

#### Response

```json
{
  "transaction_history": {
    "txns": [],
    "count": 123
  }
}
```

