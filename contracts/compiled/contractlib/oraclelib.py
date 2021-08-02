import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Oracle(Contract):
    def __init__(self, label, contract='oracle.wasm.gz', admin='a', uploader='a', gas='10000000', backend='test',
                 instantiated_contract=None):
        init_msg = json.dumps({})
        super().__init__(contract, init_msg, label, admin, uploader, gas, backend,
                         instantiated_contract=instantiated_contract)

    def get_silk_price(self):
        """
        Get current silk price
        :return:
        """
        msg = json.dumps(
            {"get_scrt_price": {}})

        return self.query(msg)