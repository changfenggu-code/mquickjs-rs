#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/time.h>

#include "mquickjs.h"

static JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
static JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

#include "mqjs_stdlib.h"

static void js_log_func(void *opaque, const void *buf, size_t buf_len)
{
    (void)opaque;
    fwrite(buf, 1, buf_len, stderr);
}

static void dump_error(JSContext *ctx)
{
    JSValue obj = JS_GetException(ctx);
    JS_PrintValueF(ctx, obj, JS_DUMP_LONG);
    fputc('\n', stderr);
}

static uint8_t *load_file(const char *filename, int *plen)
{
    FILE *f;
    uint8_t *buf;
    long buf_len;

    f = fopen(filename, "rb");
    if (!f) {
        perror(filename);
        exit(1);
    }
    fseek(f, 0, SEEK_END);
    buf_len = ftell(f);
    fseek(f, 0, SEEK_SET);
    buf = malloc((size_t)buf_len + 1);
    if (!buf) {
        perror("malloc");
        exit(1);
    }
    fread(buf, 1, (size_t)buf_len, f);
    buf[buf_len] = '\0';
    fclose(f);
    if (plen)
        *plen = (int)buf_len;
    return buf;
}

#if defined(__linux__) || defined(__APPLE__)
static int64_t get_time_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000 + (ts.tv_nsec / 1000000);
}

static double get_time_s(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + ((double)ts.tv_nsec / 1000000000.0);
}
#else
static int64_t get_time_ms(void)
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (int64_t)tv.tv_sec * 1000 + (tv.tv_usec / 1000);
}

static double get_time_s(void)
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (double)tv.tv_sec + ((double)tv.tv_usec / 1000000.0);
}
#endif

static JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    int i;
    JSValue v;

    (void)this_val;
    for (i = 0; i < argc; i++) {
        if (i != 0)
            putchar(' ');
        v = argv[i];
        if (JS_IsString(ctx, v)) {
            JSCStringBuf buf;
            const char *str;
            size_t len;
            str = JS_ToCStringLen(ctx, &len, v, &buf);
            fwrite(str, 1, len, stdout);
        } else {
            JS_PrintValueF(ctx, argv[i], JS_DUMP_LONG);
        }
    }
    putchar('\n');
    return JS_UNDEFINED;
}

static JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    (void)this_val;
    (void)argc;
    (void)argv;
    JS_GC(ctx);
    return JS_UNDEFINED;
}

static JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    struct timeval tv;
    (void)this_val;
    (void)argc;
    (void)argv;
    gettimeofday(&tv, NULL);
    return JS_NewInt64(ctx, (int64_t)tv.tv_sec * 1000 + (tv.tv_usec / 1000));
}

static JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    (void)this_val;
    (void)argc;
    (void)argv;
    return JS_NewInt64(ctx, get_time_ms());
}

static JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    const char *filename;
    JSCStringBuf buf_str;
    uint8_t *buf;
    int buf_len;
    JSValue ret;

    (void)this_val;
    filename = JS_ToCString(ctx, argv[0], &buf_str);
    if (!filename)
        return JS_EXCEPTION;
    buf = load_file(filename, &buf_len);
    ret = JS_Eval(ctx, (const char *)buf, (size_t)buf_len, filename, 0);
    free(buf);
    return ret;
}

static JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    (void)this_val;
    (void)argc;
    (void)argv;
    return JS_ThrowInternalError(ctx, "timers are unsupported in bench_exec_helper");
}

static JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv)
{
    (void)ctx;
    (void)this_val;
    (void)argc;
    (void)argv;
    return JS_UNDEFINED;
}

static uint8_t *compile_to_bytecode(const char *filename,
                                    const char *source,
                                    size_t source_len,
                                    size_t mem_size,
                                    uint32_t *out_len)
{
    JSContext *ctx;
    JSValue val;
    JSBytecodeHeader hdr;
    const uint8_t *data_buf;
    uint32_t data_len;
    uint8_t *mem_buf;
    uint8_t *bytecode_buf;
    size_t total_len;

    mem_buf = malloc(mem_size);
    if (!mem_buf) {
        perror("malloc");
        exit(1);
    }
    ctx = JS_NewContext2(mem_buf, mem_size, &js_stdlib, 1);
    JS_SetLogFunc(ctx, js_log_func);

    val = JS_Parse(ctx, source, source_len, filename, 0);
    if (JS_IsException(val)) {
        dump_error(ctx);
        JS_FreeContext(ctx);
        free(mem_buf);
        exit(1);
    }

    JS_PrepareBytecode(ctx, &hdr, &data_buf, &data_len, val);
    total_len = sizeof(hdr) + data_len;
    bytecode_buf = malloc(total_len);
    if (!bytecode_buf) {
        perror("malloc");
        JS_FreeContext(ctx);
        free(mem_buf);
        exit(1);
    }
    memcpy(bytecode_buf, &hdr, sizeof(hdr));
    memcpy(bytecode_buf + sizeof(hdr), data_buf, data_len);

    JS_FreeContext(ctx);
    free(mem_buf);

    *out_len = (uint32_t)total_len;
    return bytecode_buf;
}

static void run_bytecode_once(const uint8_t *template_buf,
                              uint32_t template_len,
                              size_t mem_size)
{
    JSContext *ctx;
    JSValue val;
    uint8_t *mem_buf;
    uint8_t *run_buf;

    mem_buf = malloc(mem_size);
    if (!mem_buf) {
        perror("malloc");
        exit(1);
    }
    ctx = JS_NewContext(mem_buf, mem_size, &js_stdlib);
    JS_SetLogFunc(ctx, js_log_func);

    run_buf = malloc(template_len);
    if (!run_buf) {
        perror("malloc");
        JS_FreeContext(ctx);
        free(mem_buf);
        exit(1);
    }
    memcpy(run_buf, template_buf, template_len);

    if (JS_RelocateBytecode(ctx, run_buf, template_len)) {
        fprintf(stderr, "Could not relocate bytecode\n");
        JS_FreeContext(ctx);
        free(mem_buf);
        free(run_buf);
        exit(1);
    }

    val = JS_LoadBytecode(ctx, run_buf);
    if (JS_IsException(val)) {
        dump_error(ctx);
        JS_FreeContext(ctx);
        free(mem_buf);
        free(run_buf);
        exit(1);
    }

    val = JS_Run(ctx, val);
    if (JS_IsException(val)) {
        dump_error(ctx);
        JS_FreeContext(ctx);
        free(mem_buf);
        free(run_buf);
        exit(1);
    }

    JS_FreeContext(ctx);
    free(mem_buf);
    free(run_buf);
}

static size_t parse_size(const char *label, const char *value)
{
    char *end;
    unsigned long long parsed = strtoull(value, &end, 10);
    if (*value == '\0' || *end != '\0') {
        fprintf(stderr, "invalid %s '%s'\n", label, value);
        exit(2);
    }
    return (size_t)parsed;
}

int main(int argc, char **argv)
{
    const char *filename;
    size_t iterations;
    size_t mem_size;
    uint8_t *source_buf;
    int source_len;
    uint8_t *bytecode_buf;
    uint32_t bytecode_len;
    double start;
    double elapsed;
    size_t i;

    if (argc != 4) {
        fprintf(stderr, "usage: bench_exec_helper <script-path> <iterations> <mem-bytes>\n");
        return 2;
    }

    filename = argv[1];
    iterations = parse_size("iterations", argv[2]);
    mem_size = parse_size("mem-bytes", argv[3]);
    if (iterations == 0) {
        fprintf(stderr, "iterations must be > 0\n");
        return 2;
    }

    source_buf = load_file(filename, &source_len);
    bytecode_buf = compile_to_bytecode(
        filename,
        (const char *)source_buf,
        (size_t)source_len,
        mem_size,
        &bytecode_len
    );

    start = get_time_s();
    for (i = 0; i < iterations; i++) {
        run_bytecode_once(bytecode_buf, bytecode_len, mem_size);
    }
    elapsed = get_time_s() - start;

    printf("%.6f\n", elapsed / (double)iterations);

    free(bytecode_buf);
    free(source_buf);
    return 0;
}
