use slac::{
    check_variables_and_functions, compile, execute,
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

fn execute_with_stdlib(script: &str) -> Result<Value> {
    let ast = compile(script)?;
    let mut env = StaticEnvironment::default();
    extend_environment(&mut env);
    check_variables_and_functions(&env, &ast)?;

    execute(&env, &ast)
}

fn assert_execute(left: &str, right: &str) {
    let left = execute_with_stdlib(left);
    assert!(left.is_ok());

    let right = execute_with_stdlib(right);
    assert!(right.is_ok());

    assert_eq!(left, right);
}

fn assert_bool(expected: bool, script: &str) {
    assert_eq!(Ok(Value::Boolean(expected)), execute_with_stdlib(script));
}

fn assert_str(expected: &str, script: &str) {
    assert_eq!(
        Ok(Value::String(expected.to_string())),
        execute_with_stdlib(script)
    );
}

fn assert_num(expected: f64, script: &str) {
    assert_eq!(Ok(Value::Number(expected)), execute_with_stdlib(script));
}

fn assert_err(script: &str) {
    assert!(execute_with_stdlib(script).is_err());
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
    assert_bool(
        true,
        "string_to_date('2022-07-08') + 1 = string_to_date('2022-07-09')",
    );

    assert_bool(
        true,
        "inc_month(string_to_date('2022-07-08')) = string_to_date('2022-08-08')",
    );

    assert_execute(
        "date_from_rfc3339('2023-08-27T08:30:00Z')",
        "string_to_date('2023-08-27') + string_to_time('08:30:00')",
    );

    assert_num(6.0, "day_of_week(string_to_date('2023-08-27'))"); // Sunday = 6
    assert_num(4.0, "day_of_week(string_to_date('2023-08-27') + 5)"); // Friday = 4

    assert_execute(
        "date_from_rfc3339('2023-08-27T08:30:00Z')",
        "encode_date(2023,08,27) + encode_time(8,30,0)",
    );

    assert_num(18101.5, "string_to_datetime('2019-07-24 12:00:00')");
    assert_num(18101.0, "date(string_to_datetime('2019-07-24 12:00:00'))");
    assert_num(0.5, "time(string_to_datetime('2019-07-24 12:00:00'))");
}

#[test]
fn operators_full() {
    assert_bool(
        true,
        "(true and not false) and
                 (false or true) and
                 (true xor false) and
                 (10 + 20 - 30 < 50 * 5 / 25) and
                 (10 mod 3 <= 10 div 3) and
                 (round(2.5) > 2) and
                 (7 >= 8 or 9 <> 10) and
                 ('Apple' + 'Pen' = 'ApplePen')",
    )
}

fn expensive_func(_params: &[Value]) -> NativeResult {
    panic!()
}

#[test]
fn short_circuit_bool() {
    let mut env = StaticEnvironment::default();
    env.add_function("expensive", None, 0, expensive_func);

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
fn env_remove_variable() {
    let mut env = StaticEnvironment::default();

    env.add_variable("some_var", Value::Number(42.0));
    let ast = compile("some_var = 42").unwrap();
    assert_eq!(Ok(Value::Boolean(true)), execute(&env, &ast));

    env.remove_variable("some_var");
    assert_eq!(Ok(Value::Boolean(false)), execute(&env, &ast));
}

#[test]
fn env_clear_variables() {
    let mut env = StaticEnvironment::default();

    env.add_variable("some_test", Value::Number(11.0));
    let ast = compile("some_test = 11").unwrap();
    assert_eq!(Ok(Value::Boolean(true)), execute(&env, &ast));

    env.clear_variables();
    assert_eq!(Ok(Value::Boolean(false)), execute(&env, &ast));
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
