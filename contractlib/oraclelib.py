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

    def get_price(self, symbol):
        """
        Get current coin price
        :param symbol: Coin ticker
        :return:
        """
        msg = json.dumps({'get_price': {'symbol': symbol}})

        return self.query(msg)

    def get_prices(self, symbols):

        msg = json.dumps({'get_prices': {'symbols': symbols}})

        return self.query(msg)

    def register_sswap_pair(self, pair):

        msg = json.dumps({
            'register_sswap_pair': {
                'pair': { 
                    'address': pair.address,
                    'code_hash': pair.code_hash,
                }
            }
        })

        return self.execute(msg)

