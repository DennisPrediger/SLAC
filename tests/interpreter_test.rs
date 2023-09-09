use slac::{
    compile, execute,
    std::{extend_environment, NativeResult},
    StaticEnvironment, Value,
};

fn execute_raw(script: &str) -> Option<Value> {
    let ast = compile(script).unwrap();
    let env = StaticEnvironment::default();

    execute(&env, &ast)
}

fn execute_test(script: &str) -> Value {
    execute_raw(script).unwrap_or(Value::Boolean(false))
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
    assert!(execute_raw("1 + 'some_string'").is_none());
    assert!(execute_raw("1 - 'some_string'").is_none());
    assert!(execute_raw("1 * 'some_string'").is_none());
    assert!(execute_raw("1 / 'some_string'").is_none());
    assert!(execute_raw("1 mod 'some_string'").is_none());
    assert!(execute_raw("1 div 'some_string'").is_none());
}

fn execute_with_stdlib(script: &str) -> Value {
    let ast = compile(script).unwrap();
    let mut env = StaticEnvironment::default();
    extend_environment(&mut env);

    execute(&env, &ast).unwrap()
}

#[test]
fn std_lib_max_min() {
    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib("max(10, 20) > min(50, 30, 10)")
    );

    assert_eq!(Value::Number(20.0), execute_with_stdlib("max(-30, 20)"));
    assert_eq!(Value::Number(-20.0), execute_with_stdlib("min(-20, 30)"));
}

#[test]
fn std_lib_contains() {
    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib("contains([1,2,3], 1)")
    );

    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib("contains('something', 'thing')")
    );

    assert_eq!(
        Value::Boolean(false),
        execute_with_stdlib("contains('something', 'other')")
    );
}

#[test]
fn std_lib_lowercase_uppercase() {
    assert_eq!(
        Value::String(String::from("hello world 😀")),
        execute_with_stdlib("lowercase('Hello World 😀')")
    );

    assert_eq!(
        Value::String(String::from("HELLO WORLD 😀")),
        execute_with_stdlib("uppercase('Hello World 😀')")
    );
}

#[test]
fn std_str() {
    assert_eq!(
        Value::String(String::from("99")),
        execute_with_stdlib("str(99)")
    );

    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib("str(true) = 'true'")
    );
}

#[test]
fn std_lib_full() {
    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib(
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
             "
        )
    );
}

#[test]
fn std_time() {
    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib("string_to_date('2022-07-08') + 1 = string_to_date('2022-07-09')")
    );

    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib(
            "inc_month(string_to_date('2022-07-08')) = string_to_date('2022-08-08')"
        )
    );

    assert_eq!(
        execute_with_stdlib("date_from_rfc3339('2023-08-27T08:30:00Z')"),
        execute_with_stdlib("string_to_date('2023-08-27') + string_to_time('08:30:00')")
    );

    assert_eq!(
        Value::Number(6.0), // Sunday = 6
        execute_with_stdlib("day_of_week(string_to_date('2023-08-27'))")
    );
    assert_eq!(
        Value::Number(4.0), // Friday = 4
        execute_with_stdlib("day_of_week(string_to_date('2023-08-27') + 5)")
    );

    assert_eq!(
        execute_with_stdlib("date_from_rfc3339('2023-08-27T08:30:00Z')"),
        execute_with_stdlib("encode_date(2023,08,27) + encode_time(8,30,0)")
    );

    assert_eq!(
        Value::Number(18101.5),
        execute_with_stdlib("string_to_datetime('2019-07-24 12:00:00')")
    );

    assert_eq!(
        Value::Number(18101.0),
        execute_with_stdlib("date(string_to_datetime('2019-07-24 12:00:00'))")
    );

    assert_eq!(
        Value::Number(0.5),
        execute_with_stdlib("time(string_to_datetime('2019-07-24 12:00:00'))")
    );
}

#[test]
fn operators_full() {
    assert_eq!(
        Value::Boolean(true),
        execute_with_stdlib(
            "(true and not false) and
             (false or true) and
             (true xor false) and
             (10 + 20 - 30 < 50 * 5 / 25) and
             (10 mod 3 <= 10 div 3) and
             (round(2.5) > 2) and
             (7 >= 8 or 9 <> 10) and
             ('Apple' + 'Pen' = 'ApplePen')"
        )
    );
}

fn expensive_func(_params: &[Value]) -> NativeResult {
    assert!(false);
    Ok(Value::Boolean(false))
}

#[test]
fn short_circuit_bool() {
    let mut env = StaticEnvironment::default();
    env.add_native_func("expensive", None, expensive_func);

    let ast = compile("false and expensive()").unwrap();
    let result = execute(&env, &ast);
    assert_eq!(Some(Value::Boolean(false)), result);

    let ast = compile("true or expensive()").unwrap();
    let result = execute(&env, &ast);
    assert_eq!(Some(Value::Boolean(true)), result);
}

#[test]
fn empty_var_comparison() {
    assert_eq!(
        Some(Value::Boolean(true)),
        execute_raw("does_not_exist = ''")
    );
    assert_eq!(
        Some(Value::Boolean(false)),
        execute_raw("does_not_exist <> ''")
    );
}
