import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Mint(Contract):
    def __init__(self, label, native_asset, oracle, treasury=None, commission=None, contract='mint.wasm.gz', admin='a', uploader='a',
                 backend='test', instantiated_contract=None, code_id=None):
        init_msg = {
            "native_asset": {
                "address": native_asset.address, 
                "code_hash": native_asset.code_hash,
            },
            "oracle": {
                "address": oracle.address, 
                "code_hash": oracle.code_hash
            },
        }
        if treasury:
            init_msg['treasury'] = {
                'address': treasury.address,
                'code_hash': treasury.code_hash,
            }
        if commission:
            init_msg['commission'] = str(commission)

        #print('MINT INIT', json.dumps(init_msg, indent=2))
        init_msg = json.dumps(init_msg)

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
        new_mint.contract_id = code_id
        new_mint.code_hash = code_hash
        return new_mint

    def update_config(self, owner=None, native_asset=None, oracle=None):
        """
        Updates the minting contract's config
        :param owner: New admin
        :param native_asset: Snip20 to Mint
        :param oracle: Oracle contract
        :return: Result
        """
        raw_msg = {"update_config": {}}
        if owner is not None:
            raw_msg["update_config"]["owner"] = owner
        if native_asset is not None:
            contract = {
                "address": native_asset.address,
                "code_hash": native_asset.code_hash
            }
            raw_msg["update_config"]["native_asset"] = contract
        if oracle is not None:
            contract = {
                "address": oracle.address,
                "code_hash": oracle.code_hash
            }
            raw_msg["update_config"]["oracle"] = contract

        msg = json.dumps(raw_msg)
        return self.execute(msg)

    def register_asset(self, snip20):
        """
        Registers a SNIP20 asset
        :param snip20: SNIP20 object to add
        :return: Result
        """
        msg = json.dumps(
            {"register_asset": {"contract": {"address": snip20.address, "code_hash": snip20.code_hash}}})

        return self.execute(msg)

    def update_asset(self, old_snip20, snip20):
        """
        Updates a SNIP20 asset's info
        :param old_snip20: The registered snip20
        :param snip20: New snip20 to replace with
        :return: Result
        """
        msg = json.dumps(
            {"update_asset": {"asset": old_snip20.address, "contract": {"address": snip20.address,
                                                                        "code_hash": snip20.code_hash}}})

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
