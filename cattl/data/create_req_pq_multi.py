from pyrogram.raw.functions import ReqPqMulti

obj = ReqPqMulti(nonce=123)
with open("req_pq_multi.bin", "wb") as f:
    f.write(obj.write())

