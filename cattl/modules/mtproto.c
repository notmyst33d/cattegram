#include "cattl.h"

int read_reqpqmulti(cattl_object *obj, cattl_reader *reader) {
    cattl_put(obj, "nonce", cattl_read_int128(reader));
    return 0;
}

int write_reqpqmulti(cattl_object *obj, cattl_writer *writer) {
    cattl_write_int128(writer, cattl_get(obj, "nonce"));
    return 0;
}

void cattl_extension() {
    cattl_add_handler(0xBE7E8EF1, &read_reqpqmulti, &write_reqpqmulti);
}
