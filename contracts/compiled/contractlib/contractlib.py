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
        self.hash = contracts[int(self.contract_id)-1]["data_hash"]
        self.address = initResponse["logs"][0]["events"][0]["attributes"][4]["value"]