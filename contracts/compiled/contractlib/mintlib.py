from .contractlib import Contract
from .secretlib import secretlib
import json


# TODO: add all of the functions
class Mint(Contract):
    def __init__(self, label, silk, oracle, contract='mint.wasm.gz', admin='a', uploader='a', gas='10000000',
                 backend='test'):
        initMsg = json.dumps(
            {"silk_contract": silk.address, "silk_contract_code_hash": silk.hash,
             "oracle_contract": "none", "oracle_contract_code_hash": "none"})
        super().__init__(contract, initMsg, label, admin, uploader, gas, backend)

    def register_asset(self, snip20):
        """
        Registers a SNIP20 asset
        :param snip20: SNIP20 object to add
        :return: Result
        """
        msg = json.dumps(
            {"register_asset": {"contract": snip20.address, "code_hash": snip20.hash}})

        return secretlib.execute_contract(self.address, msg, self.admin, self.backend)

    def get_asset(self, snip20):
        """
        Returns that assets info
        :param snip20: SNIP20 object to query
        :return:
        """
        msg = json.dumps(
            {"get_asset": {"contract": snip20.address}})

        return secretlib.query_contract(self.address, msg)