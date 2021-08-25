import string
import random
import base64
import json


def gen_label(length):
    # With combination of lower and upper case
    return ''.join(random.choice(string.ascii_letters) for i in range(length))


def to_base64(dict):
    dict_str = json.dumps(dict).encode('ascii')
    encoded_str = base64.b64encode(dict_str)
    return encoded_str.decode()
