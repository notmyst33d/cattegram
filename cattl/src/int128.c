#include <cattl.h>

// int128 is not really a convenient type in C, so just store it in char* instead
char *cattl_read_int128(cattl_reader *reader) {
    reader->position += 16;
    return reader->data + reader->position - 16;
}
