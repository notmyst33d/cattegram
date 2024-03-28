from pyrogram.raw.types import ResPQ

obj = ResPQ(nonce=123, server_nonce=123, pq=b"TESTING", server_public_key_fingerprints=[123, 321])
with open("resPQ.bin", "wb") as f:
    f.write(obj.write())

