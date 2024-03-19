typedef struct {
    const char *name;
    void *data;
} cattl_field;

typedef struct {
    unsigned int hash;
    cattl_field **fields;
    unsigned long fields_length;
} cattl_object;

typedef struct {
    char *data;
    unsigned long position;
} cattl_reader;

typedef struct {
    char *data;
    unsigned long size;
} cattl_writer;

typedef struct {
	void *data;
	unsigned long length;
} cattl_vector;

typedef int (*cattl_object_reader)(cattl_object *obj, cattl_reader *reader);
typedef int (*cattl_object_writer)(cattl_object *obj, cattl_writer *writer);

typedef struct {
    unsigned int hash;
	cattl_object_reader read;
	cattl_object_writer write;
} cattl_handler;

cattl_object *cattl_new(unsigned int hash);
cattl_reader *cattl_reader_new(char *data);

void cattl_add_handler(unsigned int hash, cattl_object_reader reader, cattl_object_writer writer);

void cattl_put(cattl_object *obj, const char *name, char *data);
void *cattl_get(cattl_object *obj, const char *name);

cattl_object *cattl_read(char *data);
char *cattl_write(cattl_object *obj);

int cattl_read_int_be(cattl_reader *reader);

char *cattl_read_int128(cattl_reader *reader);
