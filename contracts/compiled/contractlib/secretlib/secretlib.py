import subprocess
import json
import time

# Presetup some commands
query_list_code = ['secretcli', 'query', 'compute', 'list-code']


def run_command(command, wait=6):
    """
    Will run any cli command and return its output after waiting a set amount
    :param command: Array of command to run
    :param wait: Time to wait for command
    :return: Output string
    """

    result = subprocess.run(command, stdout=subprocess.PIPE, text=True)
    time.sleep(wait)
    return result.stdout


def store_contract(contract, user='a', gas='10000000', backend='test'):
    """
    Store contract and return its ID
    :param contract: Contract name
    :param user: User to upload with
    :param gas: Gas to use
    :param backend: Keyring backend
    :return: Contract ID
    """

    command = ['secretcli', 'tx', 'compute', 'store', contract,
               '--from', user, '--gas', gas, '-y', '--keyring-backend', backend]

    return run_command_query_hash(command, 9)['logs'][0]['events'][0]['attributes'][3]['value']


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
            user, '--label', label, '-y', '--keyring-backend', backend]

    return run_command_query_hash(command)


def list_code():

    command = ['secretcli', 'query', 'compute', 'list-code']

    return json.loads(run_command(command, 3))

#secretcli tx compute execute secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx '{"deposit": {}}' --amount 100uscrt  --from admin --gas 10000000 -y
def execute_contract(contract, msg, user='a', backend='test', amount=''):
    command = ['secretcli', 'tx', 'compute', 'execute', contract, msg, '--from', user, '--gas', '10000000', '-y',
            '--keyring-backend', backend]

    if amount != '':
        command.append("--amount")
        command.append(amount)

    return run_command_compute_hash(command)


def query_hash(hash):
    return run_command(['secretcli', 'q', 'tx', hash], 3)


def compute_hash(hash):
    return run_command(['secretcli', 'q', 'compute', 'tx', hash])


def query_contract(contract, msg):
    command = ['secretcli', 'query', 'compute', 'query', contract, msg]

    return json.loads(run_command(command))


def run_command_compute_hash(command, wait=6):
    out = run_command(command, wait)
    txhash = json.loads(out)["txhash"]
    return json.loads(compute_hash(txhash))


def run_command_query_hash(command, wait=6):
    out = run_command(command, wait)
    txhash = json.loads(out)["txhash"]
    return json.loads(query_hash(txhash))
