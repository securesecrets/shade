# lb-factory

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "admin_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "query_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "owner": "secret1...owner",
  "fee_recipient": "secret1...recipient",
  "recover_staking_funds_receiver": "secret1...fundsrecipient",
  "max_bins_per_swap": 500
}'
```

## Execute Messages

### set_lb_pair_implementation

```sh
secretcli tx compute execute secret1foobar '{
  "set_lb_pair_implementation": {
    "implementation": {
      "id": 1,
      "code_hash": "0123456789ABCDEF"
    }
  }
}'
```

### set_lb_token_implementation

```sh
secretcli tx compute execute secret1foobar '{
  "set_lb_token_implementation": {
    "implementation": {
      "id": 1,
      "code_hash": "0123456789ABCDEF"
    }
  }
}'
```

### create_lb_pair

```sh
secretcli tx compute execute secret1foobar '{
  "create_lb_pair": {
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
    "active_id": 8388608,
    "bin_step": 100,
    "viewing_key": "api_key_etc",
    "entropy": "shade rocks"
  }
}'
```

### set_pair_preset

```sh
secretcli tx compute execute secret1foobar '{
  "set_pair_preset": {
    "bin_step": 100,
    "base_factor": 100,
    "filter_period": 100,
    "decay_period": 100,
    "reduction_factor": 100,
    "variable_fee_control": 100,
    "protocol_share": 100,
    "max_volatility_accumulator": 100,
    "total_reward_bins": 10,
    "rewards_distribution_algorithm": "time_based_rewards",
    "epoch_staking_index": 1,
    "epoch_staking_duration": 100,
    "expiry_staking_duration": null,
    "is_open": true
  }
}'
```

### set_preset_open_state

```sh
secretcli tx compute execute secret1foobar '{
  "set_preset_open_state": {
    "bin_step": 100,
    "is_open": true
  }
}'
```

### remove_preset

```sh
secretcli tx compute execute secret1foobar '{
  "remove_preset": {
    "bin_step": 100
  }
}'
```

### set_fee_parameters_on_pair

```sh
secretcli tx compute execute secret1foobar '{
  "set_fee_parameters_on_pair": {
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
    "base_factor": 100,
    "filter_period": 100,
    "decay_period": 100,
    "reduction_factor": 100,
    "variable_fee_control": 100,
    "protocol_share": 100,
    "max_volatility_accumulator": 100
  }
}'
```

### set_fee_recipient

```sh
secretcli tx compute execute secret1foobar '{
  "set_fee_recipient": {
    "fee_recipient": "secret1...recipient"
  }
}'
```

### add_quote_asset

```sh
secretcli tx compute execute secret1foobar '{
  "add_quote_asset": {
    "asset": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    }
  }
}'
```

### remove_quote_asset

```sh
secretcli tx compute execute secret1foobar '{
  "remove_quote_asset": {
    "asset": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    }
  }
}'
```

### force_decay

```sh
secretcli tx compute execute secret1foobar '{
  "force_decay": {
    "pair": {
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
      "contract": {
        "address": "secret1...foobar",
        "code_hash": "0123456789ABCDEF"
      }
    }
  }
}'
```

## Query Messages with responses

### get_min_bin_step

```sh
secretcli query compute query secret1foobar '{
  "get_min_bin_step": {}
}'
```

#### Response

```json
{
  "min_bin_step": 100
}
```

### get_fee_recipient

```sh
secretcli query compute query secret1foobar '{
  "get_fee_recipient": {}
}'
```

#### Response

```json
{
  "fee_recipient": "secret1...recipient"
}
```

### get_lb_pair_implementation

```sh
secretcli query compute query secret1foobar '{
  "get_lb_pair_implementation": {}
}'
```

#### Response

```json
{
  "lb_pair_implementation": {
    "id": 1,
    "code_hash": "0123456789ABCDEF"
  }
}
```

### get_lb_token_implementation

```sh
secretcli query compute query secret1foobar '{
  "get_lb_token_implementation": {}
}'
```

#### Response

```json
{
  "lb_token_implementation": {
    "id": 1,
    "code_hash": "0123456789ABCDEF"
  }
}
```

### get_number_of_lb_pairs

```sh
secretcli query compute query secret1foobar '{
  "get_number_of_lb_pairs": {}
}'
```

#### Response

```json
{
  "lb_pair_number": 1
}
```

### get_lb_pair_at_index

```sh
secretcli query compute query secret1foobar '{
  "get_lb_pair_at_index": {
    "index": 0
  }
}'
```

#### Response

```json
{
  "lb_pair": {
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
    "contract": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    }
  }
}
```

### get_number_of_quote_assets

```sh
secretcli query compute query secret1foobar '{
  "get_number_of_quote_assets": {}
}'
```

#### Response

```json
{
  "number_of_quote_assets": 10
}
```

### get_quote_asset_at_index

```sh
secretcli query compute query secret1foobar '{
  "get_quote_asset_at_index": {
    "index": 0
  }
}'
```

#### Response

```json
{
  "asset": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  }
}
```

### is_quote_asset

```sh
secretcli query compute query secret1foobar '{
  "is_quote_asset": {
    "token": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    }
  }
}'
```

#### Response

```json
{
  "is_quote": true
}
```

### get_lb_pair_information

```sh
secretcli query compute query secret1foobar '{
  "get_lb_pair_information": {
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
    "bin_step": 100
  }
}'
```

#### Response

```json
{
  "lb_pair_information": {
    "bin_step": 100,
    "info": {
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
      "contract": {
        "address": "secret1...foobar",
        "code_hash": "0123456789ABCDEF"
      }
    },
    "created_by_owner": true,
    "ignored_for_routing": false
  }
}
```

### get_preset

```sh
secretcli query compute query secret1foobar '{
  "get_preset": {
    "bin_step": 100
  }
}'
```

#### Response

```json
{
  "base_factor": 100,
  "filter_period": 100,
  "decay_period": 100,
  "reduction_factor": 100,
  "variable_fee_control": 100,
  "protocol_share": 100,
  "max_volatility_accumulator": 100,
  "is_open": false
}
```

### get_all_bin_steps

```sh
secretcli query compute query secret1foobar '{
  "get_all_bin_steps": {}
}'
```

#### Response

```json
{
  "bin_step_with_preset": [
    20,
    50,
    100
  ]
}
```

### get_open_bin_steps

```sh
secretcli query compute query secret1foobar '{
  "get_open_bin_steps": {}
}'
```

#### Response

```json
{
  "open_bin_steps": [
    20,
    50,
    100
  ]
}
```

### get_all_lb_pairs

```sh
secretcli query compute query secret1foobar '{
  "get_all_lb_pairs": {
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
}'
```

#### Response

```json
{
  "lb_pairs_available": [
    {
      "bin_step": 100,
      "info": {
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
        "contract": {
          "address": "secret1...foobar",
          "code_hash": "0123456789ABCDEF"
        }
      },
      "created_by_owner": true,
      "ignored_for_routing": false
    },
    {
      "bin_step": 100,
      "info": {
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
        "contract": {
          "address": "secret1...foobar",
          "code_hash": "0123456789ABCDEF"
        }
      },
      "created_by_owner": true,
      "ignored_for_routing": false
    }
  ]
}
```

