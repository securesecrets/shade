import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Mint(Contract):
    def __init__(self, label, oracle, contract='mint.wasm.gz', admin='a', uploader='a',
                 backend='test', instantiated_contract=None, code_id=None):
        init_msg = json.dumps(
            {"oracle": {"address": oracle.address, "code_hash": oracle.code_hash}})
        super().__init__(contract, init_msg, label, admin, uploader, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

    def migrate(self, label, code_id, code_hash):
        """
        Instantiate another mint contract and migrate this contracts info into that one
        :param label: Label name of the contract
        :param code_id: Code id of the contract
        :param code_hash: Code hash
        :return: new Mint
        """
        msg = json.dumps(
            {"migrate": {"label": label, "code_id": code_id, "code_hash": code_hash}})

        new_mint = copy.deepcopy(self)
        for attribute in self.execute(msg, compute=False)["logs"][0]["events"][0]["attributes"]:
            if attribute["key"] == "contract_address":
                new_mint.address = attribute["value"]
                break
        new_mint.code_id = code_id
        new_mint.code_hash = code_hash
        return new_mint

    def update_config(self, owner=None, oracle=None):
        """
        Updates the minting contract's config
        :param owner: New admin
        :param silk:  Silk contract
        :param oracle: Oracle contract
        :return: Result
        """
        raw_msg = {"update_config": {}}
        if owner is not None:
            raw_msg["update_config"]["owner"] = owner
        if oracle is not None:
            contract = {
                "address": oracle.address,
                "code_hash": oracle.code_hash
            }
            raw_msg["update_config"]["oracle"] = contract

        msg = json.dumps(raw_msg)
        return self.execute(msg)

    def register_asset(self, snip20, name=None, burnable=None, total_burned=None):
        """
        Registers a SNIP20 asset
        :param total_burned: Total value burned
        :param burnable: If burning is allowed
        :param name: The Snip20's ticker
        :param snip20: SNIP20 object to add
        :return: Result
        """
        raw_msg = {"register_asset": {"contract": {"address": snip20.address, "code_hash": snip20.code_hash}}}
        if name is not None:
            raw_msg["register_asset"]["name"] = name
        if burnable is not None:
            raw_msg["register_asset"]["burnable"] = burnable
        if total_burned is not None:
            raw_msg["register_asset"]["total_burned"] = total_burned
        msg = json.dumps(raw_msg)

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
