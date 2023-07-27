from .secretlib import secretlib



import json


class PreInstantiatedContract:
    def __init__(self, address, code_hash, code_id):
        self.address = address
        self.code_hash = code_hash
        self.code_id = code_id

    def execute(self, msg, sender, amount=None, compute=True):
        """
        Execute said msg
        :param msg: Execute msg
        :param sender: Who will be sending the message, defaults to contract admin
        :param amount: Optional string amount to send along with transaction
        :return: Result
        """
        return secretlib.execute_contract(self.address, msg, sender, 'test',  amount, compute)

    def query(self, msg):
        """
        Query said msg
        :param msg: Query msg
        :return: Query
        """
        return secretlib.query_contract(self.address, msg)

    def as_dict(self):
        return {'address': self.address, 'code_hash': self.code_hash }


class Contract:
    def __init__(self, contract, initMsg, label, admin='a', uploader='a', backend='test',
                 instantiated_contract=None, code_id=None):
        self.label = label
        self.admin = admin
        self.uploader = uploader
        self.backend = backend

        if code_id:
            self.code_id = code_id
        else:
            self.code_id = secretlib.store_contract(contract, uploader, backend)

        if instantiated_contract:
            self.code_id = instantiated_contract.code_id
            self.address = instantiated_contract.address
            self.code_hash = instantiated_contract.code_hash
        else:
            Response = secretlib.instantiate_contract(str(self.code_id), initMsg, label, admin, backend)

            try:
                for log in Response['logs']:
                    for event in log['events']:
                        for attribute in event["attributes"]:
                            if attribute["key"] == "contract_address":
                                self.address = attribute["value"]
                                break
                self.code_hash = secretlib.contract_hash(self.address)
            except Exception as e:
                print(Response)
                raise e
                
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
              f"Id:      {self.code_id}\n"
              f"Hash:    {self.code_hash}")

    def as_dict(self):
        return {'address': self.address, 'code_hash': self.code_hash }

