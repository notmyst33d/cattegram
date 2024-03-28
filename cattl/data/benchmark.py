import time
from io import BytesIO
from multiprocessing import Process, Value
from pyrogram.raw.core import TLObject

WAIT = 1.0

def benchmark(i, data):
    while True:
        TLObject.read(data)
        data.seek(0)
        i.value += 1

with open("data.bin", "rb") as f:
    data = BytesIO(f.read())

i = Value("i", 0)
p = Process(target=benchmark, args=(i, data))
p.start()
start = time.time()

while start >= time.time() - WAIT:
    pass

p.terminate()

print(f"Decoded {i.value} objects in {round(time.time() - start, 2)} second(s)")
