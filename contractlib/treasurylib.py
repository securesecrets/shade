import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Treasury(Contract):
    def __init__(self, label, viewing_key='password', contract='treasury.wasm.gz', admin='drpresident', uploader='drpresident',
                 backend='test', instantiated_contract=None, code_id=None):
        init_msg = json.dumps({
            'viewing_key': viewing_key,
        })

        super().__init__(contract, init_msg, label, admin, uploader, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

    def get_balance(self, contract):
        return self.query(json.dumps({
            'get_balance': {
                'contract': contract.address,
            }
        }))['balance']['amount']


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

    def get_config(self):
        """
        Get the contracts config information
        :return: Contract config info
        """
        msg = json.dumps(
            {"get_config": {}})

        return self.query(msg)
