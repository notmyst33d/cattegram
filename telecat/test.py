import logging
logging.basicConfig(level=logging.DEBUG)

from pyrogram import Client
from pyrogram.session.internals.data_center import DataCenter
from pyrogram.crypto.rsa import PublicKey, server_public_keys
from typing import Tuple

def DataCenter__new__(cls, dc_id: int, test_mode: bool, ipv6: bool, media: bool) -> Tuple[str, int]:
    return "127.0.0.1", 8443

DataCenter.__new__ = DataCenter__new__

server_public_keys[0xc3b42b026ce86b21 - (1 << 64)] = PublicKey(
    int(
        "C150023E2F70DB7985DED064759CFECF0AF328E69A41DAF4D6F01B538135A6F9"
        "1F8F8B2A0EC9BA9720CE352EFCF6C5680FFC424BD634864902DE0B4BD6D49F4E"
        "580230E3AE97D95C8B19442B3C0A10D8F5633FECEDD6926A7F6DAB0DDB7D457F"
        "9EA81B8465FCD6FFFEED114011DF91C059CAEDAF97625F6C96ECC74725556934"
        "EF781D866B34F011FCE4D835A090196E9A5F0E4449AF7EB697DDB9076494CA5F"
        "81104A305B6DD27665722C46B60E5DF680FB16B210607EF217652E60236C255F"
        "6A28315F4083A96791D7214BF64C1DF4FD0DB1944FB26A2A57031B32EEE64AD1"
        "5A8BA68885CDE74A5BFC920F6ABF59BA5C75506373E7130F9042DA922179251F",
        16
    ),
    int("010001", 16)
)

client = Client(":memory:", 1234, "aaaa", in_memory=True)
client.start()
