# lb_pair

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "factory": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "token_x": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  },
  "token_y": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  },
  "bin_step": 100,
  "pair_parameters": {
    "base_factor": 5000,
    "filter_period": 30,
    "decay_period": 600,
    "reduction_factor": 5000,
    "variable_fee_control": 40000,
    "protocol_share": 1000,
    "max_volatility_accumulator": 350000
  },
  "active_id": 8388608,
  "lb_token_implementation": {
    "id": 0,
    "code_hash": ""
  },
  "staking_contract_implementation": {
    "id": 0,
    "code_hash": ""
  },
  "viewing_key": "viewing_key",
  "entropy": "entropy",
  "protocol_fee_recipient": "secret1...fundsrecipient",
  "admin_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "total_reward_bins": 10,
  "rewards_distribution_algorithm": "time_based_rewards",
  "epoch_staking_index": 1,
  "epoch_staking_duration": 100,
  "expiry_staking_duration": null,
  "recover_staking_funds_receiver": "secret1...fundsrecipient"
}'
```

## Execute Messages

### add_liquidity

```sh
secretcli tx compute execute secret1foobar '{
  "add_liquidity": {
    "liquidity_parameters": {
      "token_x": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "token_y": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "bin_step": 100,
      "amount_x": "110",
      "amount_y": "110",
      "amount_x_min": "110",
      "amount_y_min": "110",
      "active_id_desired": 8388608,
      "id_slippage": 1000,
      "delta_ids": [
        -5,
        -4,
        -3,
        -2,
        -1,
        0,
        1,
        2,
        3,
        4,
        5
      ],
      "distribution_x": [
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10
      ],
      "distribution_y": [
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10,
        10
      ],
      "deadline": 1701283067
    }
  }
}'
```

### remove_liquidity

```sh
secretcli tx compute execute secret1foobar '{
  "remove_liquidity": {
    "remove_liquidity_params": {
      "token_x": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "token_y": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "bin_step": 100,
      "amount_x_min": "10",
      "amount_y_min": "10",
      "ids": [
        8388608
      ],
      "amounts": [
        "10"
      ],
      "deadline": 1701283067
    }
  }
}'
```

### swap_tokens

```sh
secretcli tx compute execute secret1foobar '{
  "swap_tokens": {
    "offer": {
      "token": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "amount": "100"
    },
    "expected_return": null,
    "to": "secret1...recipient",
    "padding": null
  }
}'
```

### swap_tokens_invoke

```sh
secretcli tx compute execute secret1foobar '{
  "swap_tokens": {
    "expected_return": null,
    "to": "secret1...recipient",
    "padding": null
  }
}'
```

### collect_protocol_fees

```sh
secretcli tx compute execute secret1foobar '{
  "collect_protocol_fees": {}
}'
```

### increase_oracle_length

```sh
secretcli tx compute execute secret1foobar '{
  "increase_oracle_length": {
    "new_length": 100
  }
}'
```

### set_static_fee_parameters

```sh
secretcli tx compute execute secret1foobar '{
  "set_static_fee_parameters": {
    "base_factor": 5000,
    "filter_period": 30,
    "decay_period": 600,
    "reduction_factor": 5000,
    "variable_fee_control": 40000,
    "protocol_share": 1000,
    "max_volatility_accumulator": 350000
  }
}'
```

### force_decay

```sh
secretcli tx compute execute secret1foobar '{
  "force_decay": {}
}'
```

### calculte_rewards

```sh
secretcli tx compute execute secret1foobar '{
  "calculate_rewards": {}
}'
```

### reset_rewards_config

```sh
secretcli tx compute execute secret1foobar '{
  "reset_rewards_config": {
    "distribution": "time_based_rewards",
    "base_rewards_bins": 20
  }
}'
```

### set_contract_status

```sh
secretcli tx compute execute secret1foobar '{
  "set_contract_status": {
    "contract_status": "freeze_all"
  }
}'
```

## Query Messages with responses

### get_staking_contract

```sh
secretcli query compute query secret1foobar '{
  "get_staking_contract": {}
}'
```

#### Response

```json
{
  "contract": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}
```

### get_lb_token

```sh
secretcli query compute query secret1foobar '{
  "get_lb_token": {}
}'
```

#### Response

```json
{
  "contract": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}
```

### get_pair_info

```sh
secretcli query compute query secret1foobar '{
  "get_pair_info": {}
}'
```

#### Response

```json
{
  "liquidity_token": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "factory": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "pair": {
    "token_0": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    },
    "token_1": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    }
  },
  "amount_0": "12345",
  "amount_1": "12345",
  "total_liquidity": "4200876899744891917384329470959789995640432360000000",
  "contract_version": 1,
  "fee_info": {
    "shade_dao_address": "secret1...recipient",
    "lp_fee": {
      "nom": 10000000,
      "denom": 1000
    },
    "shade_dao_fee": {
      "nom": 10000000,
      "denom": 1000
    },
    "stable_lp_fee": {
      "nom": 10000000,
      "denom": 1000
    },
    "stable_shade_dao_fee": {
      "nom": 10000000,
      "denom": 1000
    }
  },
  "stable_info": {
    "stable_params": {
      "a": "10",
      "gamma1": "4",
      "gamma2": "6",
      "oracle": {
        "address": "ORACLE",
        "code_hash": "oracle_hash"
      },
      "min_trade_size_x_for_y": "0.000000001",
      "min_trade_size_y_for_x": "0.000000001",
      "max_price_impact_allowed": "500",
      "custom_iteration_controls": null
    },
    "stable_token0_data": {
      "oracle_key": "oracle_key",
      "decimals": 8
    },
    "stable_token1_data": {
      "oracle_key": "oracle_key",
      "decimals": 8
    },
    "p": "123"
  }
}
```

### swap_simulation

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
    "exclude_fee": true
  }
}'
```

#### Response

```json
{
  "total_fee_amount": "100",
  "lp_fee_amount": "90",
  "shade_dao_fee_amount": "10",
  "result": {
    "return_amount": "100000"
  },
  "price": "42008768657166552252904831246223292524636112144"
}
```

### get_factory

```sh
secretcli query compute query secret1foobar '{
  "get_factory": {}
}'
```

#### Response

```json
{
  "factory": "secret1...foobar"
}
```

### get_tokens

```sh
secretcli query compute query secret1foobar '{
  "get_tokens": {}
}'
```

#### Response

```json
{
  "token_x": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  },
  "token_y": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  }
}
```

### get_token_x

```sh
secretcli query compute query secret1foobar '{
  "get_token_x": {}
}'
```

#### Response

```json
{
  "token_x": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  }
}
```

### get_token_y

```sh
secretcli query compute query secret1foobar '{
  "get_token_y": {}
}'
```

#### Response

```json
{
  "token_y": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  }
}
```

### get_bin_step

```sh
secretcli query compute query secret1foobar '{
  "get_bin_step": {}
}'
```

#### Response

```json
{
  "bin_step": 100
}
```

### get_reserves

```sh
secretcli query compute query secret1foobar '{
  "get_reserves": {}
}'
```

#### Response

```json
{
  "reserve_x": 1000,
  "reserve_y": 1000
}
```

### get_active_id

```sh
secretcli query compute query secret1foobar '{
  "get_active_id": {}
}'
```

#### Response

```json
{
  "active_id": 8388608
}
```

### get_bin_reserves

```sh
secretcli query compute query secret1foobar '{
  "get_bin_reserves": {
    "id": 8388608
  }
}'
```

#### Response

```json
{
  "bin_id": 8388608,
  "bin_reserve_x": 1000,
  "bin_reserve_y": 1000
}
```

### get_bins_reserves

```sh
secretcli query compute query secret1foobar '{
  "get_bins_reserves": {
    "ids": [
      8388607,
      8388608,
      8388609
    ]
  }
}'
```

#### Response

```json
[
  {
    "bin_id": 8388607,
    "bin_reserve_x": 1000,
    "bin_reserve_y": 0
  },
  {
    "bin_id": 8388608,
    "bin_reserve_x": 1000,
    "bin_reserve_y": 1000
  },
  {
    "bin_id": 8388609,
    "bin_reserve_x": 0,
    "bin_reserve_y": 1000
  }
]
```

### get_all_bins_reserves

```sh
secretcli query compute query secret1foobar '{
  "get_all_bins_reserves": {
    "id": null,
    "page": null,
    "page_size": null
  }
}'
```

#### Response

```json
{
  "reserves": [
    {
      "bin_id": 8388607,
      "bin_reserve_x": 1000,
      "bin_reserve_y": 0
    },
    {
      "bin_id": 8388608,
      "bin_reserve_x": 1000,
      "bin_reserve_y": 1000
    },
    {
      "bin_id": 8388609,
      "bin_reserve_x": 0,
      "bin_reserve_y": 1000
    }
  ],
  "last_id": 8388609,
  "current_block_height": 123456
}
```

### get_updated_bin_at_height

```sh
secretcli query compute query secret1foobar '{
  "get_updated_bin_at_height": {
    "height": 100
  }
}'
```

#### Response

```json
[
  {
    "bin_id": 8388607,
    "bin_reserve_x": 1000,
    "bin_reserve_y": 0
  },
  {
    "bin_id": 8388608,
    "bin_reserve_x": 1000,
    "bin_reserve_y": 1000
  },
  {
    "bin_id": 8388609,
    "bin_reserve_x": 0,
    "bin_reserve_y": 1000
  }
]
```

### get_updated_bin_at_multiple_heights

```sh
secretcli query compute query secret1foobar '{
  "get_updated_bin_at_multiple_heights": {
    "heights": [
      100,
      200
    ]
  }
}'
```

#### Response

```json
[
  {
    "bin_id": 8388607,
    "bin_reserve_x": 1000,
    "bin_reserve_y": 0
  },
  {
    "bin_id": 8388608,
    "bin_reserve_x": 1000,
    "bin_reserve_y": 1000
  },
  {
    "bin_id": 8388609,
    "bin_reserve_x": 0,
    "bin_reserve_y": 1000
  }
]
```

### get_updated_bin_after_height

```sh
secretcli query compute query secret1foobar '{
  "get_updated_bin_after_height": {
    "height": 100,
    "page": 1,
    "page_size": 100
  }
}'
```

#### Response

```json
{
  "bins": [
    {
      "bin_id": 8388607,
      "bin_reserve_x": 1000,
      "bin_reserve_y": 0
    },
    {
      "bin_id": 8388608,
      "bin_reserve_x": 1000,
      "bin_reserve_y": 1000
    },
    {
      "bin_id": 8388609,
      "bin_reserve_x": 0,
      "bin_reserve_y": 1000
    }
  ],
  "current_block_height": 123456
}
```

### get_bin_updating_heights

```sh
secretcli query compute query secret1foobar '{
  "get_bin_updating_heights": {
    "page": 1,
    "page_size": 100
  }
}'
```

#### Response

```json
[
  123454,
  123455
]
```

### get_next_non_empty_bin

```sh
secretcli query compute query secret1foobar '{
  "get_next_non_empty_bin": {
    "swap_for_y": true,
    "id": 1
  }
}'
```

#### Response

```json
{
  "next_id": 8388609
}
```

### get_protocol_fees

```sh
secretcli query compute query secret1foobar '{
  "get_protocol_fees": {}
}'
```

#### Response

```json
{
  "protocol_fee_x": 1000,
  "protocol_fee_y": 1000
}
```

### get_static_fee_parameters

```sh
secretcli query compute query secret1foobar '{
  "get_static_fee_parameters": {}
}'
```

#### Response

```json
{
  "base_factor": 5000,
  "filter_period": 30,
  "decay_period": 600,
  "reduction_factor": 5000,
  "variable_fee_control": 40000,
  "protocol_share": 1000,
  "max_volatility_accumulator": 350000
}
```

### get_variable_fee_parameters

```sh
secretcli query compute query secret1foobar '{
  "get_variable_fee_parameters": {}
}'
```

#### Response

```json
{
  "volatility_accumulator": 0,
  "volatility_reference": 0,
  "id_reference": 0,
  "time_of_last_update": 0
}
```

### get_oracle_parameters

```sh
secretcli query compute query secret1foobar '{
  "get_oracle_parameters": {}
}'
```

#### Response

```json
{
  "sample_lifetime": 120,
  "size": 10,
  "active_size": 5,
  "last_updated": 1703403384,
  "first_timestamp": 1703403383
}
```

### get_oracle_sample_at

```sh
secretcli query compute query secret1foobar '{
  "get_oracle_sample_at": {
    "look_up_timestamp": 1234567890
  }
}'
```

#### Response

```json
{
  "cumulative_id": 100,
  "cumulative_volatility": 200,
  "cumulative_bin_crossed": 50
}
```

### get_price_from_id

```sh
secretcli query compute query secret1foobar '{
  "get_price_from_id": {
    "id": 8388608
  }
}'
```

#### Response

```json
{
  "price": "42008768657166552252904831246223292524636112144"
}
```

### get_id_from_price

```sh
secretcli query compute query secret1foobar '{
  "get_id_from_price": {
    "price": "42008768657166552252904831246223292524636112144"
  }
}'
```

#### Response

```json
{
  "id": 8388608
}
```

### get_swap_in

```sh
secretcli query compute query secret1foobar '{
  "get_swap_in": {
    "amount_out": "100000",
    "swap_for_y": true
  }
}'
```

#### Response

```json
{
  "amount_in": "1000",
  "amount_out_left": "10",
  "fee": "10"
}
```

### get_swap_out

```sh
secretcli query compute query secret1foobar '{
  "get_swap_out": {
    "amount_in": "100000",
    "swap_for_y": true
  }
}'
```

#### Response

```json
{
  "amount_in_left": "1000",
  "amount_out": "10",
  "total_fees": "100",
  "shade_dao_fees": "90",
  "lp_fees": "10"
}
```

### total_supply

```sh
secretcli query compute query secret1foobar '{
  "total_supply": {
    "id": 1
  }
}'
```

#### Response

```json
{
  "total_supply": "4200876899744891917384329470959789995640432360000000"
}
```

### get_rewards_distribution

```sh
secretcli query compute query secret1foobar '{
  "get_rewards_distribution": {
    "epoch_id": 1
  }
}'
```

#### Response

```json
{
  "distribution": {
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
  }
}
```

