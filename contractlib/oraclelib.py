import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Oracle(Contract):
    def __init__(self, label, band_contract, sscrt, contract='oracle.wasm.gz', admin='a', uploader='a', backend='test',
                 instantiated_contract=None, code_id=None):

        init_msg = json.dumps({
            'band': {
                'address': band_contract.address,
                'code_hash': band_contract.code_hash,
            },
            'sscrt': {
                'address': sscrt.address,
                'code_hash': sscrt.code_hash,
            }
        })

        super().__init__(contract, init_msg, label, admin, uploader, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

    def price(self, symbol):
        """
        Get current coin price
        :param symbol: Coin ticker
        :return:
        """
        msg = json.dumps({'price': {'symbol': symbol}})

        return self.query(msg)

    def register_sswap_pair(self, pair):
        msg = json.dumps({'pair': {
            'address': pair.address,
            'code_hash': pair.code_hash,
        }})

        return self.execute(msg)

    def register_index(self, symbol, basket: list):
        msg = json.dumps({
            'register_index': {
                'symbol': symbol,
                'basket': basket,
            }
        })
        print(msg)

        return self.execute(msg)
