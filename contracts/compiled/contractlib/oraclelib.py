import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Oracle(Contract):
    def __init__(self, label, band_contract, contract='oracle.wasm.gz', admin='a', uploader='a', gas='10000000', backend='test',
                 instantiated_contract=None, code_id=None):

        init_msg = json.dumps({
            'band': {
                'address': band_contract.address,
                'code_hash': band_contract.code_hash,
            }
        })

        super().__init__(contract, init_msg, label, admin, uploader, gas, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

    def get_price(self, coin):
        """
        Get current coin price
        :param coin: Coin ticker
        :return:
        """
        msg = json.dumps({'get_price': {'symbol': coin}})

        return self.query(msg)

    def get_shade_price(self):
        """
        Get current shade price
        :return:
        """

        return self.get_price('SHD')

    def get_scrt_price(self):
        """
        Get current scrt price
        :return:
        """

        return self.get_price('SCRT')
