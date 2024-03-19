#include <cattl.h>

int read_reqpqmulti(cattl_object *obj, cattl_reader *reader) {
    cattl_put(obj, "nonce", cattl_read_int128(reader));
    return 0;
}

void cattl_ext_mtproto() {
    cattl_add_handler(0xBE7E8EF1, &read_reqpqmulti, 0);
}
