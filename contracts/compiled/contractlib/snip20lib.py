from .contractlib import Contract
from .secretlib import secretlib
import json


class SNIP20(Contract):
    def __init__(self, label, name="token", symbol="TKN", decimals=3, seed="cGFzc3dvcmQ=", public_total_supply=False,
                 enable_deposit=False, enable_redeem=False, enable_mint=False, enable_burn=False,
                 contract='snip20.wasm.gz', admin='a', uploader='a', gas='10000000', backend='test',
                 instantiated_contract=None, code_id=None):
        self.view_key = ""
        initMsg = json.dumps(
            {"name": name, "symbol": symbol, "decimals": decimals, "prng_seed": seed, "config": {
                "public_total_supply": public_total_supply, "enable_deposit": enable_deposit,
                "enable_redeem": enable_redeem, "enable_mint": enable_mint, "enable_burn": enable_burn
            }})
        super().__init__(contract, initMsg, label, admin, uploader, gas, backend,
                         instantiated_contract=instantiated_contract, code_id=code_id)

    def set_minters(self, accounts):
        """
        Sets minters
        :param accounts: Accounts list
        :return: Response
        """
        msg = json.dumps(
            {"set_minters": {"minters": accounts}})

        return self.execute(msg)

    def deposit(self, account, amount):
        """
        Deposit a specified amount to contract
        :param account: User which will deposit
        :param amount: uSCRT
        :return: Response
        """
        msg = json.dumps(
            {"deposit": {}})

        return self.execute(msg, account, amount)

    def mint(self, recipient, amount):
        """
        Mint an amount into the recipients wallet
        :param recipient: Address to be minted in
        :param amount: Amount to mint
        :return: Response
        """
        msg = json.dumps(
            {"mint": {"recipient": recipient, "amount": str(amount)}})

        return self.execute(msg)

    def send(self, account, recipient, amount, message=None):
        """
        Send amount from an account to a recipient
        :param account: User to generate the key for
        :param recipient: Address to be minted in
        :param amount: Amount to mint
        :param message: Base64 encoded message
        :return: Response
        """

        raw_msg = {"send": {"recipient": recipient, "amount": str(amount)}}

        if message is not None:
            raw_msg["send"]["msg"] = message

        msg = json.dumps(raw_msg)

        return self.execute(msg, account)

    def set_view_key(self, account, entropy):
        """
        Generate view key to query balance
        :param account: User to generate the key for
        :param entropy: Password generation entropy
        :return: Password
        """
        msg = json.dumps(
            {"create_viewing_key": {"entropy": entropy}})

        return json.loads(self.execute(msg, account)["output_data_as_string"])["create_viewing_key"]["key"]

    def get_balance(self, address, password):
        """
        Gets amount of coins in wallet
        :param address: Account to access
        :param password: View key
        :return: Response
        """
        msg = json.dumps(
            {"balance": {"key": password, "address": address}})

        return self.query(msg)["balance"]["amount"]
