import subprocess
import json
import time


def run_command(command):
    wait = 6
    result = subprocess.run(command, stdout=subprocess.PIPE)
    time.sleep(wait)
    return result.stdout.decode('utf-8')


def run_command_with_output(command):
    hash = json.loads(run_command(command))["txhash"]
    return json.loads(run_command(compute_hash(hash)))


# Presetup some commands
query_list_code = ['secretcli', 'query', 'compute', 'list-code']


def store_contract(contract, user='a', gas='10000000', backend='test'):
    return ['secretcli', 'tx', 'compute', 'store', contract,
            '--from', user, '--gas', gas, '-y', '--keyring-backend', backend]


def instantiate_contract(contract, msg, label, user='a', backend='test'):
    return ['secretcli', 'tx', 'compute', 'instantiate', contract, msg, '--from',
            user, '--label', label, '-y', '--keyring-backend', backend]


def execute_contract(contract, msg, user='a', backend='test'):
    return ['secretcli', 'tx', 'compute', 'execute', contract, msg, '--from', user, '--gas', '10000000', '-y', '--keyring-backend', backend]


def compute_hash(hash):
    return ['secretcli', 'q', 'compute', 'tx', hash]


def query_contract(contract, msg):
    return ['secretcli', 'query', 'compute', 'query', contract, msg]


def get_contract_address(contract_code):
    return ['secretcli', 'query', 'compute', 'list-contract-by-code', contract_code]


# Import contracts
print("\n\nUploading contracts")
run_command(store_contract('mint.wasm.gz', 'd'))
time.sleep(3)
mint_id = json.loads(run_command(query_list_code))[0]["id"]
mint_hash = json.loads(run_command(query_list_code))[0]["data_hash"]
print(f"Mint contract:{mint_id} hash:{mint_hash}")

run_command(store_contract('snip20.wasm.gz', 'd'))
time.sleep(3)
snip20_id = json.loads(run_command(query_list_code))[1]["id"]
snip20_hash = json.loads(run_command(query_list_code))[1]["data_hash"]
print(f"SNIP20 contract:{snip20_id} hash:{snip20_hash}")

account = run_command(['secretcli', 'keys', 'show', '-a', 'a']).rstrip()

# Instantiate contracts
print("\n\nInstantiating contracts")
msg = '{"name":"token", "symbol":"TKN","decimals":3, "prng_seed":"cGFzc3dvcmQ=", "config": {"enable_mint":true}}'
run_command(instantiate_contract(str(snip20_id), msg, '"TEST-COIN"'))
test_snip20_contract = json.loads(run_command(get_contract_address('2')))[0]["address"]
print(f"Test SNIP20 contract {test_snip20_contract}")

msg = '{"name":"shade", "symbol":"SHD", "decimals":3, "prng_seed":"cGFzc3dvcmQ=", "config":{"enable_mint": true}}'
run_command(instantiate_contract(str(snip20_id), msg, '"SHADE-COIN"'))
shade_contract = json.loads(run_command(get_contract_address('2')))[1]["address"]
print(f"Shade contract {shade_contract}")

msg = '{"silk_contract": "' + shade_contract + '", "silk_contract_code_hash": "' + snip20_hash + \
      '", "oracle_contract": "none", "oracle_contract_code_hash": "nano"}'
run_command(instantiate_contract(str(mint_id), msg, '"SHADE-MINTER"'))
mint_contract = json.loads(run_command(get_contract_address('1')))[0]["address"]
print(f"Mint contract {mint_contract}")

# Add allowed minters
print("\n\nAdd allowed minters")
print(run_command_with_output(execute_contract(test_snip20_contract, '{"set_minters": {"minters": ["' + account + '"]}}'))["output_data_as_string"])
print(run_command_with_output(execute_contract(shade_contract, '{"set_minters": {"minters": ["' + mint_contract + '"]}}'))["output_data_as_string"])

# Add supported burn token
print("\n\nAdd test as supported burn contract")
msg = '{"register_asset" : {"contract": "' + test_snip20_contract + '", "code_hash": "' + snip20_hash + '"}}'
print(run_command_with_output(execute_contract(mint_contract, msg))["output_log"])
print(run_command(query_contract(mint_contract, '{"get_supported_assets":{}}')))

# Mint to user
msg = '{"mint": {"recipient": "' + account + '", "amount": "1000"}}'
print(run_command_with_output(execute_contract(test_snip20_contract, msg))["output_data_as_string"])

# Create viewing keys
print("\n\nCreating SNIP20 viewing keys")
msg = '{"create_viewing_key": {"entropy": "test"}}'
test_key_out = run_command_with_output(execute_contract(test_snip20_contract, msg))
test_key = json.loads(test_key_out["output_data_as_string"])["create_viewing_key"]["key"]
print(f"Viewing key: {test_key}")

msg = '{"balance": {"key": "' + test_key + '", "address": "' + account + '"}}'
print(run_command(query_contract(test_snip20_contract, msg)))

print("\n\nCreating shade viewing keys")
msg = '{"create_viewing_key": {"entropy": "test"}}'
shade_key_out = run_command_with_output(execute_contract(shade_contract, msg))
shade_key = json.loads(shade_key_out["output_data_as_string"])["create_viewing_key"]["key"]
print(f"Viewing key: {shade_key}")

msg = '{"balance": {"key": "' + shade_key + '", "address": "' + account + '"}}'
print(run_command(query_contract(shade_contract, msg)))

# Try send
print("\n\nSend to random contract")
randomAcc = run_command(['secretcli', 'keys', 'show', '-a', 'b']).rstrip()
msg = '{"send": {"recipient": "' + randomAcc + '", "amount": "100"}}'
print(run_command_with_output(execute_contract(test_snip20_contract, msg))["output_data_as_string"])
msg = '{"balance": {"key": "' + test_key + '", "address": "' + account + '"}}'
print(f"Test snip amount: {run_command(query_contract(test_snip20_contract, msg))}")

print("\n\nSend to mint contract")
msg = '{"send": {"recipient": "' + mint_contract + '", "amount": "100"}}'
print(run_command_with_output(execute_contract(test_snip20_contract, msg)))

msg = '{"balance": {"key": "' + test_key + '", "address": "' + account + '"}}'
print(f"Test snip amount: {run_command(query_contract(test_snip20_contract, msg))}")

msg = '{"balance": {"key": "' + shade_key + '", "address": "' + account + '"}}'
print(f"Shade amount: {run_command(query_contract(shade_contract, msg))}")
