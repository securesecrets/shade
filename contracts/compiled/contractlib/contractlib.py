from .secretlib import secretlib
import json


class PreInstantiatedContract:
    def __init__(self, address, code_hash):
        self.address = address
        self.code_hash = code_hash


class Contract:
    def __init__(self, contract, initMsg, label, admin='a', uploader='a', gas='10000000', backend='test',
                 instantiated_contract=None, code_id=None, debug=False):
        self.label = label
        self.admin = admin
        self.uploader = uploader
        self.gas = gas
        self.backend = backend
        self.debug = debug

        if debug:
            self.debug_log = {"log": []}

        if instantiated_contract is None:
            if code_id is None:
                stored_contract = secretlib.store_contract(contract, uploader, gas, backend)
                self.try_log("upload", "contract upload", stored_contract)
                for attribute in stored_contract['logs'][0]['events'][0]['attributes']:
                    if attribute["key"] == "code_id":
                        self.contract_id = attribute['value']
            else:
                self.contract_id = code_id
            initResponse = secretlib.instantiate_contract(str(self.contract_id), initMsg, label, admin, backend)
            self.try_log("instantiate", "contract instantiation", initResponse)
            contracts = secretlib.list_code()
            self.code_hash = contracts[int(self.contract_id) - 1]["data_hash"]
            for attribute in initResponse["logs"][0]["events"][0]["attributes"]:
                if attribute["key"] == "contract_address":
                    self.address = attribute["value"]
                    break

        else:
            self.contract_id = code_id
            self.code_hash = instantiated_contract.code_hash
            self.address = instantiated_contract.address

    def execute(self, msg, sender=None, amount=None, title="execute", desc="contract execution", return_log=False):
        """
        Execute said msg
        :param return_log: Return log info
        :param desc: Descriptive information of the logged info
        :param title: The title of the handle msg, used for logging
        :param msg: Execute msg
        :param sender: Who will be sending the message, defaults to contract admin
        :param amount: Optional string amount to send along with transaction
        :return: Result
        """
        signer = sender if sender is not None else self.admin
        out = secretlib.execute_contract(self.address, msg, signer, self.backend, amount)
        self.try_log(title, desc, out[1])
        if return_log:
            return out
        return out[0]

    def query(self, msg):
        """
        Query said msg
        :param msg: Query msg
        :return: Query
        """
        return secretlib.query_contract(self.address, msg)

    def try_log(self, title, desc, query):
        """
        Helper function for logging info
        :param title: The logs title
        :param desc: Better description of the log
        :param query: The queried txhash
        :return:
        """
        if self.debug:
            self.debug_log["log"].append({"action": title, "desc": desc, "gas_used": query["gas_used"]})

    def save_log(self, file_name=None):
        """
        Stores the debug log in a file
        :return:
        """

        if file_name is None:
            file_name = f"{self.label}-debug-log.json"

        with open(file_name, 'w', encoding='utf-8') as json_file:
            json.dump(self.debug_log, json_file, ensure_ascii=False, indent=4)

    def try_reset_logs(self):
        """
        Resets the logs
        :return:
        """
        if self.debug:
            self.debug_log = {"log": []}

    def print(self):
        """
        Prints the contract info
        :return:
        """
        print(f"Label:   {self.label}\n"
              f"Address: {self.address}\n"
              f"Id:      {self.contract_id}\n"
              f"Hash:    {self.code_hash}")
