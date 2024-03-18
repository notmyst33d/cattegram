#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <dlfcn.h>

#include "cattl.h"

static cattl_handler **handlers = NULL;
static unsigned long handlers_length = 0;

void cattl_put(cattl_object *obj, const char *name, char *data) {
    cattl_field *field = malloc(sizeof(cattl_field));
    field->name = name;
    field->data = data;
    obj->fields_length += 1;
    obj->fields = realloc(obj->fields, sizeof(cattl_field) * obj->fields_length);
    obj->fields[obj->fields_length - 1] = field;
}

void *cattl_get(cattl_object *obj, const char *name) {
    for (int i = 0; i < obj->fields_length; i++) {
        if (strcmp(obj->fields[i]->name, name) == 0) {
            return obj->fields[i]->data;
        }
    }
    return NULL;
}

cattl_object *cattl_new(unsigned int hash) {
    cattl_object *obj = malloc(sizeof(cattl_object));
    obj->fields_length = 0;
    obj->fields = NULL;
    obj->hash = hash;
    return obj;
}

cattl_reader *cattl_reader_new(char *data) {
    cattl_reader *reader = malloc(sizeof(cattl_reader));
    reader->data = data;
    reader->position = 0;
    return reader;
}

void cattl_add_handler(unsigned int hash, cattl_object_reader reader, cattl_object_writer writer) {
    cattl_handler *handler = malloc(sizeof(cattl_handler));
	handler->hash = hash;
	handler->read = reader;
    handler->write = writer;

	handlers_length += 1;
	handlers = realloc(handlers, sizeof(cattl_handler) * handlers_length);
	handlers[handlers_length - 1] = handler;
}

int cattl_read_int_be(cattl_reader *reader) {
    int value =
        (reader->data[reader->position] & 0xFF) |
        (reader->data[reader->position + 1] & 0xFF) << 8 |
        (reader->data[reader->position + 2] & 0xFF) << 16 |
        (reader->data[reader->position + 3] & 0xFF) << 24;
    reader->position += 4;
    return value;
}

cattl_object *cattl_read(char *data) {
    cattl_reader reader = { .data = data, .position = 0 };

    unsigned int hash = cattl_read_int_be(&reader);
    cattl_object *obj = cattl_new(hash);

    for (int i = 0; i < handlers_length; i++) {
        if (handlers[i]->hash == hash) {
            handlers[i]->read(obj, &reader);
        }
    }

    return obj;
}

// int128 is not really a convenient type in C, so just store it in char* instead
char *cattl_read_int128(cattl_reader *reader) {
    reader->position += 16;
    return reader->data + reader->position - 16;
}
