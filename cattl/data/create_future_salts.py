from pyrogram.raw.types import FutureSalts, FutureSalt

obj = FutureSalts(req_msg_id=123, now=123, salts=[FutureSalt(valid_since=123, valid_until=123, salt=123), FutureSalt(valid_since=321, valid_until=321, salt=321)])
with open("future_salts.bin", "wb") as f:
    f.write(obj.write())

