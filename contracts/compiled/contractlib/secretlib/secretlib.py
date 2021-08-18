from subprocess import Popen, PIPE
import json
import time

# Presetup some commands
query_list_code = ['secretcli', 'query', 'compute', 'list-code']
MAX_TRIES = 10


def run_command(command):
    """
    Will run any cli command and return its output after waiting a set amount
    :param command: Array of command to run
    :param wait: Time to wait for command
    :return: Output string
    """

    p = Popen(command, stdout=PIPE, stderr=PIPE, text=True)
    output, err = p.communicate()
    status = p.wait()
    return output


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
               '--from', user, '--gas', gas, '-y']

    if backend is not None:
        command += ['--keyring-backend', backend]

    return run_command_query_hash(command)


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
               user, '--gas', '300000', '--label', label, '-y']

    if backend is not None:
        command += ['--keyring-backend', backend]

    return run_command_query_hash(command)


def list_code():
    command = ['secretcli', 'query', 'compute', 'list-code']

    return json.loads(run_command(command))


def execute_contract(contract, msg, user='a', backend='test', amount=None):
    command = ['secretcli', 'tx', 'compute', 'execute', contract, msg, '--from', user, '--gas', '10000000', '-y']

    if backend is not None:
        command += ['--keyring-backend', backend]

    if amount is not None:
        command.append("--amount")
        command.append(amount)

    return run_command_query_compute_hash(command)


def query_hash(txhash):
    return run_command(['secretcli', 'q', 'tx', txhash])


def compute_hash(txhash):
    return run_command(['secretcli', 'q', 'compute', 'tx', txhash])


def query_contract(contract, msg):
    command = ['secretcli', 'query', 'compute', 'query', contract, msg]

    return json.loads(run_command(command))


def run_command_compute_hash(command):
    out = run_command(command)
    txhash = json.loads(out)["txhash"]

    for _ in range(MAX_TRIES):
        try:
            out = json.loads(compute_hash(txhash))
            return out
        except:
            time.sleep(1)
    print(' '.join(command), f'exceeded max tries ({MAX_TRIES})')


def run_command_query_hash(command):
    out = run_command(command)
    txhash = json.loads(out)["txhash"]

    for _ in range(MAX_TRIES):
        try:
            out = json.loads(query_hash(txhash))
            return out
        except:
            time.sleep(1)
    print(' '.join(command), f'exceeded max tries ({MAX_TRIES})')


def run_command_query_compute_hash(command):
    out = run_command(command)
    txhash = json.loads(out)["txhash"]

    for _ in range(MAX_TRIES):
        try:
            compute = json.loads(compute_hash(txhash))
            query = json.loads(query_hash(txhash))
            return [compute, query]
        except:
            time.sleep(1)
    print(' '.join(command), f'exceeded max tries ({MAX_TRIES})')
