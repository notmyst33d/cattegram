#include <cattl.h>

int cattl_read_int_be(cattl_reader *reader) {
    int value =
        (reader->data[reader->position] & 0xFF) |
        (reader->data[reader->position + 1] & 0xFF) << 8 |
        (reader->data[reader->position + 2] & 0xFF) << 16 |
        (reader->data[reader->position + 3] & 0xFF) << 24;
    reader->position += 4;
    return value;
}
