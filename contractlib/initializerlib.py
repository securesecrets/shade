from .contractlib import Contract
import json


class Initializer(Contract):
    def __init__(self, label, snip20_id, snip20_code_hash, silk_label, silk_seed, silk_initial_balances,
                 shade_label, shade_seed, shade_initial_balances, contract='initializer.wasm.gz',
                 admin='a', uploader='a',
                 backend='test', instantiated_contract=None, code_id=None):
        init_msg = {
            "snip20_id": int(snip20_id),
            "snip20_code_hash": snip20_code_hash,
            "shade": {
                "label": shade_label,
                "prng_seed": shade_seed,
                "initial_balances": shade_initial_balances
            },
            "silk": {
                "label": silk_label,
                "prng_seed": silk_seed,
                "initial_balances": silk_initial_balances
            },
        }

        init_msg = json.dumps(init_msg)

        super().__init__(contract, init_msg, label, admin, uploader, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

    def get_contracts(self):
        msg = json.dumps({
            "contracts": {}
        })

        return self.query(msg)