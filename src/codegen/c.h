#include <errno.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef int status_t;

typedef double value_t;

typedef struct {
  const char *data;
  int owned;
  int len;
  int refs;
} string_source_t;

typedef struct {
  string_source_t strings[100];
  value_t values[1000];
  int value_count;
} interpreter_state_t;

static interpreter_state_t STATE = {0};

static const status_t OK = 0;

static const status_t STACK_UNDERFLOW = 101;
static const status_t STACK_OVERFLOW = 102;
static const status_t STRING_MAX = 103;
static const status_t TYPE_MISMATCH = 104;
static const status_t ASSERT_FAILED = 105;

static const status_t NOT_IMPLEMENTED = 201;
static const status_t DATA_CORRUPTED = 202;
static const status_t STRING_TOO_LONG = 203;
static const status_t STDIN_FAILED = 204;

static char SCRATCH_A[100];
static char SCRATCH_B[100];

static const uint64_t TRUE_BYTES = 0x7fffa00000000000L;
static const uint64_t FALSE_BYTES = 0x7fffb00000000000L;

#define TRUE_V (*(value_t *)&TRUE_BYTES)
#define FALSE_V (*(value_t *)&FALSE_BYTES)

#define assert_stack_has(x)                                                    \
  if (STATE.value_count < x) {                                                 \
    return STACK_UNDERFLOW;                                                    \
  }

#define assert_stack_capacity(x)                                               \
  if ((sizeof(STATE.values) / sizeof(STATE.values[0])) <                       \
      STATE.value_count + x)                                                   \
    return STACK_OVERFLOW;

#define checked(x)                                                             \
  do {                                                                         \
    int r = x;                                                                 \
    if (r != 0) {                                                              \
      return r;                                                                \
    }                                                                          \
  } while (0);

#define stack_read(x, offset)                                                  \
  value_t x = STATE.values[STATE.value_count + offset];

#define stack_read_number(x, offset)                                           \
  value_t x;                                                                   \
  checked(resolve_number_value(STATE.values[STATE.value_count + offset], &x));

#define stack_read_string(x, offset)                                           \
  string_source_t *x;                                                          \
  checked(resolve_string_value(STATE.values[STATE.value_count + offset], &x));

#define stack_at(offset) (STATE.values[STATE.value_count + offset])

value_t string_index_to_value(uint64_t string_index) {
  string_index |= (0x7fff9ULL << 44);
  value_t v = *(value_t *)&string_index;
  return v;
}

value_t string_source_to_value(const string_source_t *source) {
  return string_index_to_value(source - STATE.strings);
}

void maybe_resolve_string_value(value_t v, string_source_t **target) {
  if (isnan(v)) {
    uint64_t s = *(uint64_t *)&v;
    if (s >> 44 != 0x7fff9ULL) {
      *target = NULL;
    } else {
      s &= 0xFFFFF;
      *target = STATE.strings + s;
    }
  } else {
    *target = NULL;
  }
}

status_t resolve_string_value(value_t v, string_source_t **target) {
  maybe_resolve_string_value(v, target);
  if (*target == NULL) {
    return TYPE_MISMATCH;
  }
  return OK;
}

status_t resolve_number_value(value_t v, double *target) {
  if (isnan(v)) {
    uint64_t s = *(uint64_t *)&v;

    if (s == TRUE_BYTES || s == FALSE_BYTES) {
      return TYPE_MISMATCH;
    }

    if (s >> 48 == 0x7ff9ULL) {
      return TYPE_MISMATCH;
    }
  }

  *target = v;
  return OK;
}

void inc_ref_count(value_t v) {
  string_source_t *source;
  maybe_resolve_string_value(v, &source);
  if (source != NULL) {
    source->refs++;
  }
}

void dec_string_ref_count(string_source_t *source) {
  source->refs--;
  if (source->refs == 0) {
    if (source->owned) {
      free((char *)source->data);
      source->data = NULL;
      return;
    } else {
      source->data = NULL;
    }
  }
}

void dec_ref_count(value_t v) {
  string_source_t *source;
  maybe_resolve_string_value(v, &source);
  if (source != NULL) {
    dec_string_ref_count(source);
  }
}

int strings_equal(string_source_t *first, string_source_t *second) {
  if (first == second) {
    return 1;
  }

  if (first->len != second->len) {
    return 0;
  }

  if (first->data == second->data) {
    return 1;
  }

  for (int i = 0; i < first->len; i++) {
    if (first->data[i] != second->data[i]) {
      return 0;
    }
  }
  return 1;
}

int values_equal(value_t first, value_t second) {
  if (isnan(first)) {
    string_source_t *first_string;
    maybe_resolve_string_value(first, &first_string);

    string_source_t *second_string;
    maybe_resolve_string_value(second, &second_string);

    if (first_string != NULL) {
      if (second_string == NULL) {
        return TYPE_MISMATCH;
      }
      return strings_equal(first_string, second_string);
    } else if (second_string != NULL) {
      return TYPE_MISMATCH;
    }

    uint64_t first_bytes = *(uint64_t *)&first;
    uint64_t second_bytes = *(uint64_t *)&second;

    value_t r = strings_equal(first_string, second_string) ? TRUE_V : FALSE_V;

    return (first_bytes == TRUE_BYTES && second_bytes == TRUE_BYTES) ||
           (first_bytes == FALSE_BYTES && second_bytes == FALSE_BYTES);
  }

  return first == second;
}

int is_truthy(value_t v) {
  if (isnan(v)) {
    uint64_t s = *(uint64_t *)&v;

    if (TRUE_BYTES == s) {
      return 1;
    }

    string_source_t *source;
    maybe_resolve_string_value(v, &source);
    if (source != NULL) {
      return source->len > 0;
    }
    return 0;
  } else {
    return v != 0l;
  }
}

status_t find_string_source_slot(uint64_t *target) {
  uint64_t max = sizeof(STATE.strings) / sizeof(STATE.strings[0]);
  for (uint64_t i = 0; i < max; i++) {
    string_source_t *s = STATE.strings + i;
    if (s->data == NULL) {
      *target = i;
      return OK;
    }
  }
  return STRING_MAX;
}

status_t check_condition(int *truthy) {
  assert_stack_has(1);
  stack_read(v, -1);
  STATE.value_count--;
  *truthy = is_truthy(v);
  dec_ref_count(v);
  return OK;
}

status_t print_to_string(value_t v, char *scratch_string, int scratch_length,
                         const char **result_string, int *result_length) {
  if (isnan(v)) {
    string_source_t *string_source;
    maybe_resolve_string_value(v, &string_source);
    if (string_source != NULL) {
      *result_string = string_source->data;
      *result_length = string_source->len;
      return OK;
    }

    uint64_t s = *(uint64_t *)&v;
    if (s == TRUE_BYTES) {
      *result_length = 4;
      *result_string = "true";
      return OK;
    }

    if (s == FALSE_BYTES) {
      *result_length = 5;
      *result_string = "false";
      return OK;
    }
  }

  int written_length = snprintf(scratch_string, scratch_length, "%.8g", v);
  if (written_length >= scratch_length) {
    return STRING_TOO_LONG;
  }

  *result_length = written_length;
  *result_string = scratch_string;
  return OK;
}

status_t print_stack() {
  if (STATE.value_count == 0) {
    return OK;
  }
  printf("[");
  for (int i = 0; i < STATE.value_count; i++) {
    if (i > 0) {
      printf(", ");
    }
    int len;
    const char *str;
    checked(print_to_string(STATE.values[i], SCRATCH_A, sizeof(SCRATCH_A), &str,
                            &len));
    printf("%.*s", len, str);
  }
  printf("]\n");
  return OK;
}

status_t dup() {
  assert_stack_has(1);
  assert_stack_capacity(1);
  stack_read(v, -1);
  stack_at(0) = v;
  inc_ref_count(v);
  STATE.value_count++;
  return OK;
}

status_t swap() {
  assert_stack_has(2);
  stack_read(first, -2);
  stack_at(-2) = stack_at(-1);
  stack_at(-1) = first;
  return OK;
}

status_t over() {
  assert_stack_has(2);
  assert_stack_capacity(1);
  stack_read(v, -2);
  stack_at(0) = v;
  inc_ref_count(v);
  STATE.value_count++;
  return OK;
}

status_t and_i() {
  assert_stack_has(2);
  stack_read(first, -2);
  stack_read(second, -1);
  int use_first = !is_truthy(first);
  stack_at(-2) = use_first ? first : second;
  dec_ref_count(use_first ? second : first);
  STATE.value_count--;
  return OK;
}

status_t or_i() {
  assert_stack_has(2);
  stack_read(first, -2);
  stack_read(second, -1);
  value_t r;
  int use_first = is_truthy(first);
  stack_at(-2) = use_first ? first : second;
  dec_ref_count(use_first ? second : first);
  STATE.value_count--;
  return OK;
}

status_t rot() {
  assert_stack_has(3);
  value_t first = stack_at(-3);
  stack_at(-3) = stack_at(-2);
  stack_at(-2) = stack_at(-1);
  stack_at(-1) = first;
  return OK;
}

status_t greater() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = first > second ? TRUE_V : FALSE_V;
  STATE.value_count--;
  return OK;
}

status_t less() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = first < second ? TRUE_V : FALSE_V;
  STATE.value_count--;
  return OK;
}

status_t modulo() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = fmod(first, second);
  STATE.value_count--;
  return OK;
}

status_t not() {
  assert_stack_has(1);
  stack_read(v, -1);
  stack_at(-1) = is_truthy(v) ? FALSE_V : TRUE_V;
  dec_ref_count(v);
  return OK;
}

status_t minus() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = first - second;
  STATE.value_count--;
  return OK;
}

status_t plus() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = first + second;
  STATE.value_count--;
  return OK;
}

status_t times() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = first * second;
  STATE.value_count--;
  return OK;
}

status_t divide() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = first / second;
  STATE.value_count--;
  return OK;
}

status_t pow_i() {
  assert_stack_has(2);
  stack_read_number(first, -2);
  stack_read_number(second, -1);
  stack_at(-2) = powf(first, second);
  STATE.value_count--;
  return OK;
}

status_t equals() {
  assert_stack_has(2);
  stack_read(first, -2);
  stack_read(second, -1);
  value_t r = values_equal(first, second) ? TRUE_V : FALSE_V;
  stack_at(-2) = r;
  STATE.value_count--;
  return OK;
}

status_t drop() {
  assert_stack_has(1);
  dec_ref_count(stack_at(-1));
  STATE.value_count--;
  return OK;
}

status_t length() {
  assert_stack_has(1);
  stack_read_string(source, -1);
  stack_at(-1) = source->len;
  dec_string_ref_count(source);
  return OK;
}

status_t substring() {
  assert_stack_has(3);
  stack_read_string(source, -3);
  stack_read_number(start_double, -2);
  stack_read_number(end_double, -1);

  int start = (int)start_double;
  if (start < 0) {
    start = 0;
  } else if (start >= source->len) {
    start = source->len;
  }

  int end = (int)end_double;
  if (end < start) {
    end = start;
  } else if (end >= source->len) {
    end = source->len;
  }

  int len = end - start;

  if (source->refs == 1 && (start == 0 || len == 0)) {
    STATE.value_count -= 2;
    source->len = len;
    return OK;
  }

  uint64_t string_index;
  checked(find_string_source_slot(&string_index));
  string_source_t *res = STATE.strings + string_index;

  res->len = len;
  res->owned = 1;
  res->refs = 1;

  char *data = malloc(len);
  memcpy(data, source->data + start, len);
  res->data = data;

  dec_string_ref_count(source);
  stack_at(-3) = string_index_to_value(string_index);
  STATE.value_count -= 2;
  return OK;
}

status_t join() {
  assert_stack_has(2);
  stack_read(first, -2);
  stack_read(second, -1);

  int first_len;
  const char *first_str;
  checked(print_to_string(first, SCRATCH_A, sizeof(SCRATCH_A), &first_str,
                          &first_len));

  int second_len;
  const char *second_str;
  checked(print_to_string(second, SCRATCH_B, sizeof(SCRATCH_B), &second_str,
                          &second_len));

  int len = first_len + second_len;

  uint64_t string_index;
  checked(find_string_source_slot(&string_index));
  string_source_t *res = STATE.strings + string_index;

  char *data = malloc(len);
  memcpy(data, first_str, first_len);
  memcpy(data + first_len, second_str, second_len);

  res->data = data;
  res->len = len;
  res->owned = 1;
  res->refs = 1;

  stack_at(-2) = string_index_to_value(string_index);

  dec_ref_count(first);
  dec_ref_count(second);
  STATE.value_count--;
  return OK;
}

status_t increment() {
  assert_stack_has(1);
  stack_read_number(v, -1);
  STATE.values[STATE.value_count - 1] = v + 1;
  return OK;
}

status_t decrement() {
  assert_stack_has(1);
  stack_read_number(v, -1);
  STATE.values[STATE.value_count - 1] = v - 1;
  return OK;
}

status_t assert() {
  assert_stack_has(2);
  stack_read(v, -2);
  stack_read_string(string_source, -1);
  if (!is_truthy(v)) {
    printf("Assertion failed: %.*s\n", string_source->len, string_source->data);
    return ASSERT_FAILED;
  }
  dec_ref_count(v);
  dec_string_ref_count(string_source);
  STATE.value_count -= 2;
  return OK;
}

status_t print() {
  assert_stack_has(1);
  value_t v = STATE.values[STATE.value_count - 1];
  const char *str;
  int len;
  checked(print_to_string(v, SCRATCH_A, sizeof(SCRATCH_A), &str, &len));
  printf("%.*s\n", len, str);
  dec_ref_count(v);
  STATE.value_count--;
  return OK;
}

status_t push_number_literal(value_t v) {
  assert_stack_capacity(1);
  stack_at(0) = v;
  STATE.value_count++;
  return OK;
}

status_t push_true_literal() {
  assert_stack_capacity(1);
  stack_at(0) = TRUE_V;
  STATE.value_count++;
  return OK;
}

status_t push_false_literal() {
  assert_stack_capacity(1);
  stack_at(0) = FALSE_V;
  STATE.value_count++;
  return OK;
}

status_t push_string_literal(const char *data, size_t len) {
  assert_stack_capacity(1);

  uint64_t string_index;
  checked(find_string_source_slot(&string_index));

  string_source_t *s = STATE.strings + string_index;
  s->data = data;
  s->len = len;
  s->refs = 1;
  s->owned = 0;

  stack_at(0) = string_index_to_value(string_index);
  STATE.value_count++;
  return OK;
}

status_t readline() {
  char *line = NULL;
  size_t buffer_size = 0;
  ssize_t len = getline(&line, &buffer_size, stdin);
  int err = errno;

  if (len == -1) {
    free(line);
    if (err) {
      return STDIN_FAILED;
    }
    checked(push_string_literal("", 0));
    checked(push_false_literal());
    return OK;
  }

  uint64_t string_index;
  checked(find_string_source_slot(&string_index));
  string_source_t *res = STATE.strings + string_index;

  if (len > 0 && line[len - 1] == '\n') {
    len--;
  }

  res->data = line;
  res->len = len;
  res->owned = 1;
  res->refs = 1;

  stack_at(0) = string_index_to_value(string_index);
  STATE.value_count++;

  checked(push_true_literal());
  return OK;
}
