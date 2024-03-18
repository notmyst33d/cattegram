from pyrogram.raw.functions import ReqPqMulti

obj = ReqPqMulti(nonce=123)
with open("data.bin", "wb") as f:
    f.write(obj.write())

