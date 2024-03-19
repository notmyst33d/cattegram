#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

#include <cattl.h>
#include <cattl_ext.h>

int main(void) {
	cattl_ext_mtproto();

    FILE *f = fopen("data/data.bin", "rb");
    fseek(f, 0, SEEK_END);
    long length = ftell(f);
    char *data = malloc(length);
    fseek(f, 0, SEEK_SET);
    fread(data, length, 1, f);

    cattl_object *obj = cattl_read(data);
	if (obj == NULL || obj->fields_length == 0) {
		printf("obj is empty\n");
		return 1;
	}

    char *nonce = cattl_get(obj, "nonce");
    for (int i = 0; i < 16; i++) {
        printf("%02X", nonce[i]);
    }
    printf("\n");

	return 0;
}
