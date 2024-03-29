import sys
from io import BytesIO
from pyrogram.raw.core import TLObject

with open(sys.argv[1], "rb") as f:
    print(TLObject.read(BytesIO(f.read())))

