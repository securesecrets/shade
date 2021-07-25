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

    def update_config(self, owner=None, silk_contract=None, silk_contract_code_hash=None,
                      oracle_contract=None, oracle_contract_code_hash=None):
        """
        Updates the minting contract's config
        :param owner: New admin
        :param silk_contract: New silk contract address
        :param silk_contract_code_hash: New silk contract hash
        :param oracle_contract: New oracle contract address
        :param oracle_contract_code_hash: New oracle contract hash
        :return: Result
        """
        raw_msg = {"update_config": {}}
        if owner is not None:
            raw_msg["update_config"]["owner"] = owner
        if silk_contract is not None:
            raw_msg["update_config"]["silk_contract"] = silk_contract
        if silk_contract_code_hash is not None:
            raw_msg["update_config"]["silk_contract_code_hash"] = silk_contract_code_hash
        if oracle_contract is not None:
            raw_msg["update_config"]["oracle_contract"] = oracle_contract
        if oracle_contract_code_hash is not None:
            raw_msg["update_config"]["oracle_contract_code_hash"] = oracle_contract_code_hash

        msg = json.dumps(raw_msg)
        return self.execute(msg)

    def register_asset(self, snip20):
        """
        Registers a SNIP20 asset
        :param snip20: SNIP20 object to add
        :return: Result
        """
        msg = json.dumps(
            {"register_asset": {"contract": snip20.address, "code_hash": snip20.hash}})

        return self.execute(msg)

    def update_asset(self, old_snip20, snip20):
        """
        Updates a SNIP20 asset's info
        :param old_snip20: The registered snip20
        :param snip20: New snip20 to replace with
        :return: Result
        """
        msg = json.dumps(
            {"update_asset": {"asset": old_snip20.address, "contract": snip20.address, "code_hash": snip20.hash}})

        return self.execute(msg)

    def get_supported_assets(self):
        """
        Get all supported asset addressed
        :return: Supported assets info
        """
        msg = json.dumps(
            {"get_supported_assets": {}})

        return self.query(msg)

    def get_config(self):
        """
        Get the contracts config information
        :return: Contract config info
        """
        msg = json.dumps(
            {"get_config": {}})

        return self.query(msg)

    def get_asset(self, snip20):
        """
        Returns that assets info
        :param snip20: SNIP20 object to query
        :return: Asset info
        """
        msg = json.dumps(
            {"get_asset": {"contract": snip20.address}})

        return self.query(msg)