from subprocess import Popen, PIPE
import json
import time

# Presetup some commands
query_list_code = ['secretcli', 'query', 'compute', 'list-code']
MAX_TRIES = 10

GAS_METRICS = []
STORE_GAS = '4000000'
GAS = '4000000'


def run_command(command):
    """
    Will run any cli command and return its output after waiting a set amount
    :param command: Array of command to run
    :param wait: Time to wait for command
    :return: Output string
    """
    #print(' '.join(command))
    p = Popen(command, stdout=PIPE, stderr=PIPE, text=True)
    output, err = p.communicate()
    status = p.wait()
    if err and not output:
        return err
    return output


def store_contract(contract, user='a', backend='test'):
    """
    Store contract and return its ID
    :param contract: Contract name
    :param user: User to upload with
    :param gas: Gas to use
    :param backend: Keyring backend
    :return: Contract ID
    """

    command = ['secretcli', 'tx', 'compute', 'store', f'./compiled/{contract}',
               '--from', user, '--gas', STORE_GAS, '-y']

    if backend is not None:
        command += ['--keyring-backend', backend]

    output = run_command_query_hash(command)
    try:
        for attribute in output['logs'][0]['events'][0]['attributes']:
            if attribute["key"] == "code_id":
                return attribute['value']
    except:
        # print(output)
        return output


def instantiate_contract(contract, msg, label, user='a', backend='test'):
    """
    Instantiates a contract
    :param contract: Contract name
    :param msg: Init msg
    :param label: Name to give to the contract
    :param user: User to instantiate with
    :param backend: Keyring backend
    :return:
    """

    command = ['secretcli', 'tx', 'compute', 'instantiate', contract, msg, '--from',
               user, '--label', label, '-y', '--gas', '500000']

    if backend is not None:
        command += ['--keyring-backend', backend]

    return run_command_query_hash(command)


def list_code():
    command = ['secretcli', 'query', 'compute', 'list-code']

    return json.loads(run_command(command))


def list_contract_by_code(code):
    command = ['secretcli', 'query', 'compute', 'list-contract-by-code', code]

    return json.loads(run_command(command))

def contract_hash(address):
    command = ['secretcli', 'query', 'compute', 'contract-hash', address]

    return run_command(command)


def execute_contract(contract, msg, user='a', backend='test', amount=None, compute=True):
    command = ['secretcli', 'tx', 'compute', 'execute', contract, json.dumps(msg), '--from', user, '--gas', GAS, '-y']

    if backend is not None:
        command += ['--keyring-backend', backend]

    if amount is not None:
        command.append("--amount")
        command.append(amount)

    if compute:
        return run_command_compute_hash(command)
    return run_command_query_hash(command)


def query_hash(hash):
    return run_command(['secretcli', 'q', 'tx', hash])


def compute_hash(hash):
    print(hash)
    return run_command(['secretcli', 'q', 'compute', 'tx', hash])


def query_contract(contract, msg):
    command = ['secretcli', 'query', 'compute', 'query', contract, json.dumps(msg)]
    out = run_command(command)
    try:
        return json.loads(out)
    except json.JSONDecodeError as e:
        print(out)
        raise e


def run_command_compute_hash(command):
    out = run_command(command)

    try:
        txhash = json.loads(out)["txhash"]
        #print(txhash)

    except Exception as e:
        # print(out)
        raise e

    for _ in range(MAX_TRIES):
        try:
            out = compute_hash(txhash)
            out = json.loads(out)
            # print(out)
            # querying hash once the hash is computed so we can check gas usage
            tx_data = json.loads(query_hash(txhash))
            # print(json.dumps(tx_data))
            # print('gas:', tx_data['gas_used'], '\t/', tx_data['gas_wanted'])
            GAS_METRICS.append({
                'want': tx_data['gas_wanted'],
                'used': tx_data['gas_used'],
                'cmd': ' '.join(command)
            })
            return out
        except json.JSONDecodeError as e:
            time.sleep(1)
    print(out)
    print(' '.join(command), f'exceeded max tries ({MAX_TRIES})')


def run_command_query_hash(command):
    out = run_command(command)
    try:
        txhash = json.loads(out)["txhash"]
    except json.JSONDecodeError as e:
        print(out)
        raise e

    for _ in range(MAX_TRIES):
        try:
            # TODO: Read the gas used and store somewhere for metrics
            out = query_hash(txhash)
            out = json.loads(out)
            # print('gas:', out['gas_used'], '\t/', out['gas_wanted'])
            GAS_METRICS.append({
                'want': out['gas_wanted'],
                'used': out['gas_used'],
                'cmd': ' '.join(command)
            })
            return out
        except json.JSONDecodeError as e:
            time.sleep(1)
    print(out)
    print(' '.join(command), f'exceeded max tries ({MAX_TRIES})')
