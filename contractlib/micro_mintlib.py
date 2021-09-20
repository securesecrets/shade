import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class MicroMint(Contract):
    def __init__(self, label, native_asset, oracle, treasury=None, 
                asset_peg=None,
                contract='micro_mint.wasm.gz', 
                admin='a', uploader='a',
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

        if asset_peg:
            init_msg['peg'] = asset_peg

        print(json.dumps(init_msg, indent=2))
        init_msg = json.dumps(init_msg)

        super().__init__(contract, init_msg, label, admin, uploader, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

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

    def register_asset(self, snip20, capture=None):
        """
        Registers a SNIP20 asset
        :param snip20: SNIP20 object to add
        :param capture: Comission for the SNIP20
        :return: Result
        """
        msg = {"register_asset": {"contract": {"address": snip20.address, "code_hash": snip20.code_hash}}}

        if capture:
            msg['register_asset']['capture'] = str(capture)

        return self.execute(json.dumps(msg))

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
