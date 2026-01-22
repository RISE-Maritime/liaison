/*
 * Test suite for FMI3 String value functions
 *
 * Tests fmi3GetString and fmi3SetString parameter transmission.
 */

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "test_harness.h"
#include "call_record.h"

/* ============================================================================
 * Test: fmi3SetString with simple string
 * ============================================================================ */
static void test_set_string_simple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3String values[] = {"Hello, World!"};

    fmi3Status status = harness->fmi3SetString(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_SET_STRING);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 5);
    assert_int_equal(rec->n_strings, 1);
    assert_string_equal(rec->string_values[0], "Hello, World!");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetString with empty string
 * ============================================================================ */
static void test_set_string_empty(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3String values[] = {""};

    fmi3Status status = harness->fmi3SetString(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_string_equal(rec->string_values[0], "");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetString with multiple strings
 * ============================================================================ */
static void test_set_string_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3String values[] = {"First", "Second", "Third"};

    fmi3Status status = harness->fmi3SetString(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->n_strings, 3);
    assert_string_equal(rec->string_values[0], "First");
    assert_string_equal(rec->string_values[1], "Second");
    assert_string_equal(rec->string_values[2], "Third");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetString with UTF-8 characters
 * ============================================================================ */
static void test_set_string_utf8(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3String values[] = {"Hello \xc3\xa9\xc3\xa0\xc3\xbc"};  /* UTF-8 for eaue with accents */

    fmi3Status status = harness->fmi3SetString(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_string_equal(rec->string_values[0], "Hello \xc3\xa9\xc3\xa0\xc3\xbc");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3SetString with special characters
 * ============================================================================ */
static void test_set_string_special_chars(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3String values[] = {"Line1\nLine2\tTabbed"};

    fmi3Status status = harness->fmi3SetString(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_string_equal(rec->string_values[0], "Line1\nLine2\tTabbed");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetString with single value
 * ============================================================================ */
static void test_get_string_single(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3String values[1];

    fmi3Status status = harness->fmi3GetString(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_STRING);
    assert_int_equal(rec->n_value_references, 1);
    assert_int_equal(rec->value_references[0], 5);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: fmi3GetString with multiple values
 * ============================================================================ */
static void test_get_string_multiple(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {0, 1, 2};
    fmi3String values[3];

    fmi3Status status = harness->fmi3GetString(instance, vr, 3, values, 3);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_int_equal(rec->type, CALL_FMI3_GET_STRING);
    assert_int_equal(rec->n_value_references, 3);

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Test: String with numeric content
 * ============================================================================ */
static void test_set_string_numeric_content(void** state) {
    TestHarness* harness = (TestHarness*)*state;

    fmi3Instance instance = harness_create_instance(harness);
    assert_non_null(instance);

    harness_clear_call_log(harness);

    fmi3ValueReference vr[] = {5};
    fmi3String values[] = {"12345.67890"};

    fmi3Status status = harness->fmi3SetString(instance, vr, 1, values, 1);
    assert_int_equal(status, fmi3OK);

    harness_read_call_log(harness);

    const CallRecord* rec = harness_get_call_record(harness, 0);
    assert_non_null(rec);
    assert_string_equal(rec->string_values[0], "12345.67890");

    harness->fmi3FreeInstance(instance);
    harness->current_instance = NULL;
}

/* ============================================================================
 * Main
 * ============================================================================ */
int main(void) {
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_set_string_simple),
        cmocka_unit_test(test_set_string_empty),
        cmocka_unit_test(test_set_string_multiple),
        cmocka_unit_test(test_set_string_utf8),
        cmocka_unit_test(test_set_string_special_chars),
        cmocka_unit_test(test_get_string_single),
        cmocka_unit_test(test_get_string_multiple),
        cmocka_unit_test(test_set_string_numeric_content),
    };

    return cmocka_run_group_tests(tests, harness_group_setup, harness_group_teardown);
}
