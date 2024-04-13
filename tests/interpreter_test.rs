use slac::{
    check_variables_and_functions, compile,
    environment::{Arity, Function},
    execute,
    optimizer::{fold_constants, transform_ternary},
    stdlib::{extend_environment, NativeResult},
    Result, StaticEnvironment, Value,
};

fn execute_raw(script: &str) -> Result<Value> {
    let ast = compile(script).unwrap();
    let env = StaticEnvironment::default();

    execute(&env, &ast)
}

fn execute_test(script: &str) -> Value {
    execute_raw(script).unwrap_or(Value::Boolean(false))
}

fn execute_with_stdlib(script: &str, optimize: bool) -> Result<Value> {
    let mut ast = compile(script)?;
    let mut env = StaticEnvironment::default();
    extend_environment(&mut env);
    check_variables_and_functions(&env, &ast)?;

    if optimize {
        ast = transform_ternary(ast);
        ast = fold_constants(ast)?;
    }

    execute(&env, &ast)
}

fn assert_execute(left: &str, right: &str) {
    let left_result = execute_with_stdlib(left, false);
    let right_result = execute_with_stdlib(right, false);

    assert_eq!(left_result, right_result);

    let left_result = execute_with_stdlib(left, true);
    let right_result = execute_with_stdlib(right, true);

    assert_eq!(left_result, right_result);
}

fn assert_bool(expected: bool, script: &str) {
    assert_eq!(
        Ok(Value::Boolean(expected)),
        execute_with_stdlib(script, false)
    );

    assert_eq!(
        Ok(Value::Boolean(expected)),
        execute_with_stdlib(script, true)
    );
}

fn assert_str(expected: &str, script: &str) {
    assert_eq!(
        Ok(Value::String(expected.to_string())),
        execute_with_stdlib(script, false)
    );

    assert_eq!(
        Ok(Value::String(expected.to_string())),
        execute_with_stdlib(script, true)
    );
}

fn assert_num(expected: f64, script: &str) {
    assert_eq!(
        Ok(Value::Number(expected)),
        execute_with_stdlib(script, false)
    );

    assert_eq!(
        Ok(Value::Number(expected)),
        execute_with_stdlib(script, true)
    );
}

fn assert_err(script: &str) {
    assert!(execute_with_stdlib(script, false).is_err());
}

#[test]
fn add_number() {
    assert_eq!(Value::Number(2.0), execute_test("1 + 1 "));
    assert_eq!(Value::Number(2.0), execute_test(" 1 + 1 "));
    assert_eq!(Value::Number(100.0), execute_test("99 + 1"));
    assert_eq!(Value::Number(100.5), execute_test("99.2 + 1.3"));
}

#[test]
fn add_string() {
    let expected = Value::String(String::from("Hello World"));
    assert_eq!(expected, execute_test("'Hello World'"));
    assert_eq!(expected, execute_test("'Hello' + ' ' + 'World'"));
    assert_eq!(expected, execute_test("'Hello ' + '' + 'World'"));
}

#[test]
fn add_unicode_string() {
    let expected = Value::String(String::from("мир приветствий"));

    assert_eq!(expected, execute_test("'мир' + ' ' + 'приветствий'"));
}

#[test]
fn boolean_and() {
    assert_eq!(Value::Boolean(false), execute_test("true and false"));
    assert_eq!(Value::Boolean(true), execute_test("true and true"));
    assert_eq!(Value::Boolean(false), execute_test("false and false"));
    assert_eq!(Value::Boolean(true), execute_test("true and true and true"));
    assert_eq!(
        Value::Boolean(false),
        execute_test("true and true and false")
    );
}

#[test]
fn boolean_or() {
    assert_eq!(Value::Boolean(true), execute_test("false or true"));
    assert_eq!(Value::Boolean(true), execute_test("true or false"));
    assert_eq!(Value::Boolean(true), execute_test("true or true"));
    assert_eq!(Value::Boolean(false), execute_test("false or false"));
}

#[test]
fn boolean_empty() {
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("'' or true"));
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("0 or true"));
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("[] or true"));
    assert_eq!(Ok(Value::Boolean(false)), execute_raw("true and ''"));
    assert_eq!(Ok(Value::Boolean(false)), execute_raw("true and 0"));
    assert_eq!(Ok(Value::Boolean(false)), execute_raw("true and []"));
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("true and '1'"));
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("true and 1"));
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("true and [1]"));
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("true and not 0"));
}

#[test]
fn boolean_xor() {
    assert_eq!(Value::Boolean(true), execute_test("true xor false"));
    assert_eq!(Value::Boolean(true), execute_test("false xor true"));
    assert_eq!(Value::Boolean(false), execute_test("true xor true"));
    assert_eq!(Value::Boolean(false), execute_test("false xor false"));
}

#[test]
fn boolean_not() {
    assert_eq!(Value::Boolean(false), execute_test("not true"));
    assert_eq!(Value::Boolean(true), execute_test("not false"));

    assert_eq!(Value::Boolean(true), execute_test("not false and true"));
    assert_eq!(Value::Boolean(false), execute_test("false or not true"));
}

#[test]
fn number_arithmetics() {
    assert_eq!(Value::Number(10.0), execute_test("5+3+2"));
    assert_eq!(Value::Number(10.0), execute_test("4+3*2"));
    assert_eq!(Value::Number(2.0), execute_test("5 div 2"));
    assert_eq!(Value::Number(1.0), execute_test("5 mod 2"));
    assert_eq!(Value::Number(2.0), execute_test("50 div 20 mod 3"));
}

#[test]
fn array_combination() {
    let expected = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
        Value::Number(40.0),
    ]);

    assert_eq!(expected, execute_test("[10, 20, 30, 40]"));
    assert_eq!(expected, execute_test("[10, 20] + [30, 40]"));
    assert_eq!(expected, execute_test("[10] + [20] + [30] + [40]"));
    assert_eq!(expected, execute_test("[10, 20] + [] + [30, 40]"));

    assert_eq!(Value::Array(vec![]), execute_test("[]"));
}

#[test]
fn invalid_operations() {
    assert!(execute_raw("1 + 'some_string'").is_err());
    assert!(execute_raw("1 - 'some_string'").is_err());
    assert!(execute_raw("1 * 'some_string'").is_err());
    assert!(execute_raw("1 / 'some_string'").is_err());
    assert!(execute_raw("1 mod 'some_string'").is_err());
    assert!(execute_raw("1 div 'some_string'").is_err());
}

#[test]
fn std_lib_max_min() {
    assert_bool(true, "max(10, 20) > min(50, 30, 10)");

    assert_num(20.0, "max(-30, 20)");
    assert_num(-20.0, "min(-20, 30)");
}

#[test]
fn std_lib_contains() {
    assert_bool(true, "contains([1,2,3], 1)");
    assert_bool(true, "contains('something', 'thing')");
    assert_bool(false, "contains('something', 'other')");
}

#[test]
fn std_lib_lowercase_uppercase() {
    assert_str("hello world 😀", "lowercase('Hello World 😀')");
    assert_str("HELLO WORLD 😀", "uppercase('Hello World 😀')");
}

#[test]
fn std_str() {
    assert_str("99", "str(99)");
    assert_bool(true, "str(true) = 'true'");
}

#[test]
fn std_lib_full() {
    assert_bool(
        true,
        "
            (abs(-11.2) = 11.2) and 
            all([true, true]) and 
            any([true, false]) and
            bool(1) and
            contains('something', 'ome') and
            empty([]) and
            (float('3.14') = 3.14) and
            (int(3.14) = 3) and
            (length('hello') = 5) and
            (lowercase('BIG WORDS') = 'big words') and
            (uppercase('small words') = 'SMALL WORDS') and
            (max(-10, 5) = 5) and
            (min(-10, 5) = -10) and
            (pow(10, 2) = 100) and
            (round(3.4) = round(2.5)) and
            (str(-10) = '-10') and
            (trim('  space   ') = 'space')
             ",
    );
}

#[test]
#[cfg(feature = "chrono")]
fn std_time() {
    assert_execute(
        "string_to_date('2022-07-08') + 1",
        "string_to_date('2022-07-09')",
    );

    assert_execute(
        "inc_month(string_to_date('2022-07-08'))",
        "string_to_date('2022-08-08')",
    );

    assert_num(19596.0, "string_to_date('2023-08-27')");
    assert_num(0.5, "string_to_time('12:00:00')");
    assert_num(19596.5, "string_to_datetime('2023-08-27 12:00:00')");

    assert_execute(
        "string_to_datetime('2023-08-27 08:30:00')",
        "string_to_date('2023-08-27') + string_to_time('08:30:00')",
    );

    assert_num(6.0, "day_of_week(string_to_date('2023-08-27'))"); // Sunday = 6
    assert_num(4.0, "day_of_week(string_to_date('2023-08-27') + 5)"); // Friday = 4

    assert_num(18101.5, "string_to_datetime('2019-07-24 12:00:00')");
    assert_num(18101.0, "date(string_to_datetime('2019-07-24 12:00:00'))");
    assert_num(0.5, "time(string_to_datetime('2019-07-24 12:00:00'))");
}

#[allow(dead_code)]
// #[test] // dependent on the local timezone
fn std_time_rfc() {
    assert_num(
        19596.5 + 1. / 24. * 2.,
        "date_from_rfc3339('2023-08-27T12:00:00+00:00')",
    );
    assert_num(19596.5, "date_from_rfc3339('2023-08-27T12:00:00+02:00')");
    assert_num(
        19596.5 + 1. / 24. * 2.,
        "date_from_rfc2822('Sun, 27 Aug 2023 12:00:00 +0000')",
    );
    assert_num(
        19596.5,
        "date_from_rfc2822('Sun, 27 Aug 2023 12:00:00 +0200')",
    );

    assert_str(
        "2023-08-27T12:00:00+02:00",
        "date_to_rfc3339(date_from_rfc3339('2023-08-27T12:00:00+02:00'))",
    );
    assert_str(
        "Sun, 27 Aug 2023 12:00:00 +0200",
        "date_to_rfc2822(date_from_rfc2822('Sun, 27 Aug 2023 12:00:00 +0200'))",
    );
}

#[test]
fn operators_full() {
    assert_bool(
        true,
        "(true and not false) and
                 (false or true) and
                 (true xor false) and
                 (10 + 20 - 30 < 50 * 5 / 25) and // 0 < 10
                 (10 mod 3 <= 10 div 3) and       // 1 <= 3
                 (round(2.5) > 2) and             // 3 > 2
                 (7 >= 8 or 9 <> 10) and          // false or true
                 ('Apple' + 'Pen' = 'ApplePen')",
    )
}

fn expensive_func(_params: &[Value]) -> NativeResult {
    panic!()
}

#[test]
fn short_circuit_bool() {
    let mut env = StaticEnvironment::default();
    env.add_function(Function::new(expensive_func, Arity::None, "expensive()"));

    let ast = compile("false and expensive()").unwrap();
    let result = execute(&env, &ast);
    assert_eq!(Ok(Value::Boolean(false)), result);

    let ast = compile("true or expensive()").unwrap();
    let result = execute(&env, &ast);
    assert_eq!(Ok(Value::Boolean(true)), result);
}

#[test]
fn empty_var_comparison() {
    assert_eq!(Ok(Value::Boolean(true)), execute_raw("does_not_exist = ''"));
    assert_eq!(
        Ok(Value::Boolean(false)),
        execute_raw("does_not_exist <> ''")
    );
}

#[test]
fn optional_params() {
    assert_bool(true, "replace('Hello', 'o', 'p') = 'Hellp'");
    assert_bool(true, "replace('Hello', 'o') = 'Hell'");
    assert_bool(true, "pow(10) = 100");
    assert_bool(true, "pow(10, 3) = 1000");
}

#[test]
#[cfg(feature = "regex")]
fn regex_is_match() {
    assert_bool(true, "re_is_match('ABCDE', 'BC')");
    assert_bool(false, "re_is_match('ABCDE', 'EF')");
}

#[test]
#[cfg(feature = "regex")]
fn regex_find() {
    assert_execute("re_find('ABCDE', 'BC')", "['BC']");
    assert_execute(
        "re_find('an employer has an employee in employment', 'employ(er|ee|ment|ing|able)')",
        "['employer', 'employee', 'employment']",
    );
    assert_execute(
        r"re_find('john.smith@example.com','([a-z0-9_\.\-]+)@([\da-z\.\-]+)\.([a-z\.]{2,5})')",
        "['john.smith@example.com']",
    );

    assert_execute(r"re_find('12354', '\D')", "[]");
    assert_execute(r"re_find('ABCDE', '\D*')", "['ABCDE']");
    assert_execute(r"re_find('ABCDE', '\D')", "['A','B','C','D','E']");
}

#[test]
#[cfg(feature = "regex")]
fn regex_capture() {
    assert_execute(
        r"re_capture('john.smith@example.com', '(.*)@(.*)\.(.*)')",
        r"['john.smith@example.com', 'john.smith', 'example', 'com']",
    );

    assert_execute(
        r"re_capture('john.smith@example', '(.*)@(.*)\.?(.*)?')",
        r"['john.smith@example', 'john.smith', 'example', '']",
    );

    assert_execute(
        r"re_capture('11 aa 22 bb', '(\d{2})\W(\D{2})')",
        r"['11 aa', '11', 'aa']",
    );

    assert_execute(r"re_capture('111', '(\D)(\D)')", r"['', '', '']");
}

#[test]
#[cfg(feature = "regex")]
fn regex_replace() {
    assert_execute(
        r"re_replace('john.smith@example.com', '(.*)@(.*)\.(.*)', '$1@test.$3')",
        r"'john.smith@test.com'",
    );

    assert_execute(r"re_replace('AAAAAA', 'A', 'B')", r"'BBBBBB'");
    assert_execute(r"re_replace('AAAAAA', 'A', 'B', 3)", r"'BBBAAA'");
}

#[test]
fn array_at() {
    assert_execute(r"at([1, 'Test', true], 0)", r"1");
    assert_execute(r"at([1, 'Test', true], 1)", r"'Test'");
    assert_execute(r"at([1, 'Test', true], 2)", r"true");

    #[cfg(feature = "regex")]
    assert_execute(
        r"at(re_capture('john.smith@example.com', '(.*)@(.*)\.(.*)'), 2)",
        r"'example'",
    );

    assert_err("at([1,2], 10)");
    assert_err("at([1,2], -1)");
}

#[cfg(not(feature = "zero_based_strings"))]
mod test_strings {
    use crate::{assert_bool, assert_err, assert_execute};

    #[test]
    fn string_at() {
        assert_execute("at('abc', 1)", "'a'");
        assert_execute("at('abc', 2)", "'b'");
        assert_err("at('123', 4)");
        assert_err("at(123, 1)");
    }

    #[test]
    fn string_find() {
        assert_execute("find('ABC', 'B')", "2");
        assert_execute("find('ABCD', 'BC')", "2");
        assert_execute("find('ABCD', 'E')", "0");
    }

    #[test]
    fn string_copy() {
        assert_bool(true, "copy('Test', 2, 2) = 'es'");
        assert_bool(true, "copy('Test', 2, 20) = 'est'");
        assert_bool(true, "copy('Test', find('Test', 'e'), 1) = 'e'");
    }
}

#[cfg(feature = "zero_based_strings")]
mod test_strings {
    use crate::{assert_bool, assert_err, assert_execute};

    #[test]
    fn string_at() {
        assert_execute("at('abc', 0)", "'a'");
        assert_execute("at('abc', 1)", "'b'");
        assert_err("at('123', 4)");
        assert_err("at(123, 1)");
    }

    #[test]
    fn string_find() {
        assert_execute("find('ABC', 'B')", "1");
        assert_execute("find('ABCD', 'BC')", "1");
        assert_execute("find('ABCD', 'E')", "-1");
    }

    #[test]
    fn string_copy() {
        assert_bool(true, "copy('Test', 1, 2) = 'es'");
        assert_bool(true, "copy('Test', 1, 20) = 'est'");
        assert_bool(true, "copy('Test', find('Test', 'e'), 1) = 'e'");
    }
}

#[test]
fn common_replace() {
    assert_execute("replace([1, 2, 3], 1, 2)", "[2, 2, 3]");
    assert_execute("replace([1, 1, 1], 1, 2)", "[2, 2, 2]");
    assert_execute("replace([3, 3, 3], 1, 2)", "[3, 3, 3]");
    assert_execute(
        "replace(['Hello', 'World'], 'Hello', 'Goodbye')",
        "['Goodbye', 'World']",
    );
    assert_execute("replace([1, 2, 3], 1)", "[2, 3]");
}

#[test]
fn common_remove() {
    assert_execute("remove([1, 2, 3], 2)", "[1, 3]");
    assert_execute("remove('Hello World', 'l')", "'Heo Word'");
    assert_err("remove([1, 2, 3], 1, 2)");
}

#[test]
fn ternary_if() {
    assert_execute("if_then(true, 1, 2)", "1");
    assert_execute("if_then(false, 1, 2)", "2");
    assert_execute("if_then(true, 1)", "1");
    assert_execute("if_then(false, 1)", "0");
}

#[test]
fn optimize_fold() {
    assert_execute("1+1", "2");
    assert_execute("1+1--2", "4");
    assert_execute("1+1--------2", "4");
    assert_execute("1 + 2 > 3 + 4", "false");
    assert_execute("if_then(1 = 2, 3, 4)", "4");
    assert_execute("if_then(max(1,3) = 2, 3, 4)", "4");
    assert_execute(
        "if_then(1 = 2, if_then(true, 1, 2), if_then(false, 3, 4))",
        "4",
    );
}

#[test]
fn has_comments() {
    assert_execute("3.14", "3 + .14 // eh, close enough");
    assert_execute("3", "3{ + .14}");
    assert_execute("3", "3{ + .14} // todo for later");
    assert_execute("8", "3{ {-} + .14} + 5");
    assert_execute("3", "3{ ");
    assert_execute(
        "8",
        "
    4
    // chosen by fair dice roll
    + 4
    ",
    );
    assert_execute(
        "8",
        "
    4
    {
    + 5
    }
    + 4
    ",
    );

    assert_execute(
        "13",
        "
    4
    //{
    + 5
    //}
    + 4
    ",
    );

    assert_err(
        "
    4
    //{
    + 5
    } // closed brace
    + 4
    ",
    );

    assert_err("// todo add expression");
    assert_err("{todo add expression}");
}
