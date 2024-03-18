#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

#include "cattl.h"

int main(void) {
	cattl_load_extension("mtproto");
    FILE *f = fopen("data/data.bin", "rb");
    fseek(f, 0, SEEK_END);
    long length = ftell(f);
    char *data = malloc(length);
    fseek(f, 0, SEEK_SET);
    fread(data, length, 1, f);

    cattl_object *obj = cattl_read(data);
    char *nonce = cattl_get(obj, "nonce");
    for (int i = 0; i < 16; i++) {
        printf("%02X", nonce[i]);
    }
    printf("\n");
}
