use slac::compile;
use slac::environment::StaticEnvironment;
use slac::interpreter::TreeWalkingInterpreter;
use slac::stdlib::add_stdlib;
use slac::value::Value;

fn execute(script: &str) -> Value {
    let ast = compile(script).unwrap();
    let env = StaticEnvironment::default();

    TreeWalkingInterpreter::interprete(&env, &ast)
}

#[test]
fn add_number() {
    assert_eq!(Value::Number(2.0), execute("1 + 1 "));
    assert_eq!(Value::Number(2.0), execute(" 1 + 1 "));
    assert_eq!(Value::Number(100.0), execute("99 + 1"));
    assert_eq!(Value::Number(100.5), execute("99.2 + 1.3"));
}

#[test]
fn add_string() {
    let expected = Value::String("Hello World".to_string());
    assert_eq!(expected, execute("'Hello World'"));
    assert_eq!(expected, execute("'Hello' + ' ' + 'World'"));
    assert_eq!(expected, execute("'Hello ' + '' + 'World'"));
}

#[test]
fn add_unicode_string() {
    let expected = Value::String("мир приветствий".to_string());

    assert_eq!(expected, execute("'мир' + ' ' + 'приветствий'"));
}

#[test]
fn boolean_and() {
    assert_eq!(Value::Boolean(false), execute("true and false"));
    assert_eq!(Value::Boolean(true), execute("true and true"));
    assert_eq!(Value::Boolean(false), execute("false and false"));
    assert_eq!(Value::Boolean(true), execute("true and true and true"));
    assert_eq!(Value::Boolean(false), execute("true and true and false"));
}

#[test]
fn boolean_or() {
    assert_eq!(Value::Boolean(true), execute("false or true"));
    assert_eq!(Value::Boolean(true), execute("true or false"));
    assert_eq!(Value::Boolean(true), execute("true or true"));
    assert_eq!(Value::Boolean(false), execute("false or false"));
}

#[test]
fn boolean_xor() {
    assert_eq!(Value::Boolean(true), execute("true xor false"));
    assert_eq!(Value::Boolean(true), execute("false xor true"));
    assert_eq!(Value::Boolean(false), execute("true xor true"));
    assert_eq!(Value::Boolean(false), execute("false xor false"));
}

#[test]
fn boolean_not() {
    assert_eq!(Value::Boolean(false), execute("not true"));
    assert_eq!(Value::Boolean(true), execute("not false"));

    assert_eq!(Value::Boolean(true), execute("not false and true"));
    assert_eq!(Value::Boolean(false), execute("false or not true"));
}

#[test]
fn number_arithmetics() {
    assert_eq!(Value::Number(10.0), execute("5+3+2"));
    assert_eq!(Value::Number(10.0), execute("4+3*2"));
    assert_eq!(Value::Number(2.0), execute("5 div 2"));
    assert_eq!(Value::Number(1.0), execute("5 mod 2"));
    assert_eq!(Value::Number(2.0), execute("50 div 20 mod 3"));
}

#[test]
fn array_combination() {
    let expected = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
        Value::Number(40.0),
    ]);

    assert_eq!(expected, execute("[10, 20, 30, 40]"));
    assert_eq!(expected, execute("[10, 20] + [30, 40]"));
    assert_eq!(expected, execute("[10] + [20] + [30] + [40]"));
    assert_eq!(expected, execute("[10, 20] + [] + [30, 40]"));

    assert_eq!(Value::Array(vec![]), execute("[]"));
}

#[test]
fn invalid_operations() {
    assert_eq!(Value::Nil, execute("1 + 'some_string'"));
    assert_eq!(Value::Nil, execute("1 - 'some_string'"));
    assert_eq!(Value::Nil, execute("1 * 'some_string'"));
    assert_eq!(Value::Nil, execute("1 / 'some_string'"));
    assert_eq!(Value::Nil, execute("1 mod 'some_string'"));
    assert_eq!(Value::Nil, execute("1 div 'some_string'"));
}

fn execute_with_stdlib(script: &str) -> Value {
    let ast = compile(script).unwrap();
    let mut env = StaticEnvironment::default();
    add_stdlib(&mut env);

    TreeWalkingInterpreter::interprete(&env, &ast)
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
        Value::String("hello world 😀".to_string()),
        execute_with_stdlib("lowercase('Hello World 😀')")
    );

    assert_eq!(
        Value::String("HELLO WORLD 😀".to_string()),
        execute_with_stdlib("uppercase('Hello World 😀')")
    );
}

#[test]
fn std_str() {
    assert_eq!(
        Value::String("99".to_string()),
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
            "(abs(-11.2) = 11.2) and 
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
             (trim('  space   ') = 'space')"
        )
    );
}

fn expensive_func(_params: &[Value]) -> Result<Value, String> {
    assert!(false);
    Ok(Value::Nil)
}

#[test]
fn short_circuit_bool() {
    let mut env = StaticEnvironment::default();
    env.add_native_func("expensive", None, expensive_func);

    let ast = compile("false and expensive()").unwrap();
    let result = TreeWalkingInterpreter::interprete(&env, &ast);
    assert_eq!(Value::Boolean(false), result);

    let ast = compile("true or expensive()").unwrap();
    let result = TreeWalkingInterpreter::interprete(&env, &ast);
    assert_eq!(Value::Boolean(true), result);
}
