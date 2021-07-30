from .secretlib import secretlib


class Contract:
    def __init__(self, contract, initMsg, label, admin='a', uploader='a', gas='10000000', backend='test', wait=6):
        self.label = label
        self.admin = admin
        self.uploader = uploader
        self.gas = gas
        self.backend = backend
        self.wait = wait

        self.contract_id = secretlib.store_contract(contract, uploader, gas, backend)
        initResponse = secretlib.instantiate_contract(str(self.contract_id), initMsg, label, admin, backend)
        contracts = secretlib.list_code()
        self.code_hash = contracts[int(self.contract_id) - 1]["data_hash"]
        for attribute in initResponse["logs"][0]["events"][0]["attributes"]:
            if attribute["key"] == "contract_address":
                self.address = attribute["value"]
                break

    def execute(self, msg, sender=None, amount=None, compute=True):
        """
        Execute said msg
        :param msg: Execute msg
        :param sender: Who will be sending the message, defaults to contract admin
        :param amount: Optional string amount to send along with transaction
        :return: Result
        """
        signer = sender if sender is not None else self.admin
        return secretlib.execute_contract(self.address, msg, signer, self.backend, amount, compute)

    def query(self, msg):
        """
        Query said msg
        :param msg: Query msg
        :return: Query
        """
        return secretlib.query_contract(self.address, msg)

    def print(self):
        """
        Prints the contract info
        :return:
        """
        print(f"Label:   {self.label}\n"
              f"Address: {self.address}\n"
              f"Id:      {self.contract_id}\n"
              f"Hash:    {self.code_hash}")
