import copy

from .contractlib import Contract
from .secretlib import secretlib
import json


class Oracle(Contract):
    def __init__(self, label, contract='oracle.wasm.gz', admin='a', uploader='a', gas='10000000', backend='test',
                 instantiated_contract=None):
        init_msg = json.dumps({
            'band': {
                'address': 'secret1p0jtg47hhwuwgp4cjpc46m7qq6vyjhdsvy2nph',
                'code_hash': '77c854ea110315d5103a42b88d3e7b296ca245d8b095e668c69997b265a75ac5',
            }
        })
        super().__init__(contract, init_msg, label, admin, uploader, gas, backend,
                         instantiated_contract=instantiated_contract)

    def get_shade_price(self):
        """
        Get current shade price
        :return:
        """
        msg = json.dumps({'get_price': {'symbol': 'SHD'}})

        return self.query(msg)

    def get_scrt_price(self):
        """
        Get current scrt price
        :return:
        """
        msg = json.dumps({'get_price': {'symbol': 'SCRT'}})

        return self.query(msg)
