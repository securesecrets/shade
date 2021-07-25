from .secretlib import secretlib


class Contract:
    def __init__(self, contract, initMsg, label, admin='a', uploader='a', gas='10000000', backend='test'):
        self.label = label
        self.admin = admin
        self.uploader = uploader
        self.gas = gas
        self.backend = backend

        self.contract_id = secretlib.store_contract(contract, uploader, gas, backend)
        initResponse = secretlib.instantiate_contract(str(self.contract_id), initMsg, label, admin, backend)
        contracts = secretlib.list_code()
        self.hash = contracts[int(self.contract_id) - 1]["data_hash"]
        self.address = initResponse["logs"][0]["events"][0]["attributes"][4]["value"]

    def execute(self, msg, sender=None, amount=None):
        """
        Execute said msg
        :param msg: Execute msg
        :param sender: Who will be sending the message, defaults to contract admin
        :param amount: Optional string amount to send along with transaction
        :return: Result
        """
        signer = sender if sender is not None else self.admin
        return secretlib.execute_contract(self.address, msg, signer, self.backend, amount)

    def query(self, msg):
        """
        Query said msg
        :param msg: Query msg
        :return: Query
        """
        return secretlib.query_contract(self.address, msg)
