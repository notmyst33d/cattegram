import hashlib

from pyrogram.raw.core.primitives.bytes import Bytes

server_key_modulus = bytes.fromhex("""
b8:2b:36:77:ac:43:5b:4d:7f:7c:99:89:2f:17:
bb:88:de:8d:e5:c0:c0:46:dd:b1:b3:16:4e:f7:f2:
33:e7:71:da:15:9c:aa:60:23:ab:df:93:8b:cd:fe:
62:28:c8:40:f0:ee:f6:ab:2f:15:35:d5:1f:a0:a6:
a4:d4:b7:0d:9c:fd:1b:b0:89:30:a3:79:87:b2:f5:
00:86:62:30:39:35:7f:8e:ba:65:ca:d5:f2:5d:c8:
69:42:cf:7c:7e:02:38:4c:75:eb:7b:cd:42:d4:e2:
1b:b3:8c:8b:ec:a2:5c:6b:ba:63:39:de:ba:e7:b1:
20:30:e0:2f:67:c9:60:19:b3:0f:84:c8:a0:df:ea:
d5:43:9a:47:24:92:e6:f4:2e:22:ab:5b:ba:7e:f3:
cb:2f:9b:2a:73:fb:39:4b:34:1c:89:7a:13:e4:3f:
11:1c:1a:48:33:ba:7a:91:2d:dc:c8:bb:9f:2d:33:
ec:6a:cb:21:fc:b2:af:e1:b6:d0:26:24:97:af:9e:
d3:4b:c0:f7:15:19:e9:76:d1:bf:3b:93:ca:51:73:
c2:04:3c:ae:b1:30:f5:42:9d:e0:d7:ff:79:d2:f7:
e3:55:a2:98:fa:fa:39:6b:53:78:4f:94:42:fb:f1:
ff:55:d2:ba:be:ac:3d:77:e0:2c:7a:e4:a6:14:85:
83:27
""".replace(":", "").replace(" ", "").replace("\n", ""))

server_key_private_exponent = bytes.fromhex("""
82:59:4b:6e:be:44:da:2d:6a:cb:14:9c:4c:f1:
7e:31:2c:5b:c0:3e:81:98:66:b6:ab:eb:61:2f:0a:
85:04:0d:c7:ee:3d:f9:34:9c:6a:bc:8e:60:88:06:
c1:10:de:53:60:73:b1:b0:8d:5d:74:c1:88:66:1b:
2c:54:c5:ce:e3:cb:1d:3b:00:8e:a5:55:91:de:ee:
9b:55:81:cf:61:de:64:8e:ef:8c:b6:58:60:f7:51:
ae:76:b4:06:07:e5:a8:55:2d:ac:e2:ed:ef:9c:5f:
00:7e:3f:3e:95:d8:80:93:5a:d3:7a:15:13:b2:82:
e2:26:d5:b5:97:be:a6:be:e8:91:a8:22:94:42:25:
7c:5d:34:21:bf:33:68:10:07:af:e9:29:d5:7c:8b:
63:fb:5b:16:c3:bf:4b:71:e0:f7:3c:63:35:34:e0:
07:50:c8:95:8e:cf:1b:15:9f:d4:11:da:9f:26:26:
6c:e8:d8:b8:7a:05:72:d7:e6:e9:d5:ef:61:55:d9:
ad:1c:6c:24:37:65:15:56:14:f3:c6:2a:e6:d0:34:
47:1e:ca:08:8e:4d:15:62:1c:9a:fa:5e:a6:0a:48:
20:ca:e4:3a:4e:17:1c:91:48:2f:9a:3b:24:a0:ee:
be:c9:90:58:51:32:91:d7:d7:17:76:67:28:d7:12:
eb:01
""".replace(":", "").replace(" ", "").replace("\n", ""))

server_key_exponent = bytes.fromhex("010001")  # Guaranteed to be 65537 with 2048 bit RSA

digest = hashlib.sha1(Bytes(server_key_modulus) + Bytes(server_key_exponent)).digest()
fingerprint = bytes(reversed(digest[len(digest) - 8:]))

print(f"0x{fingerprint.hex()}")
print(f"SHA1 digest length: {len(digest)}")
