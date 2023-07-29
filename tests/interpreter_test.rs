use slac::compile;
use slac::environment::Environment;
use slac::interpreter::TreeWalkingInterpreter;
use slac::value::Value;

fn execute(script: &str) -> Value {
    let ast = compile(script).unwrap();
    let env = Environment::new();

    TreeWalkingInterpreter::interprete(&env, &ast)
}

#[test]
fn test_add_number() {
    assert_eq!(Value::Number(100.0), execute("99 + 1"));
    assert_eq!(Value::Number(100.5), execute("99.2 + 1.3"));
}

#[test]
fn test_add_string() {
    assert_eq!(
        Value::String("Hello World".to_string()),
        execute("'Hello' + ' ' + 'World'")
    );
}

#[test]
fn test_boolean_and() {
    assert_eq!(Value::Boolean(false), execute("true and false"));
    assert_eq!(Value::Boolean(true), execute("true and true"));
    assert_eq!(Value::Boolean(false), execute("false and false"));
    assert_eq!(Value::Boolean(true), execute("true and true and true"));
    assert_eq!(Value::Boolean(false), execute("true and true and false"));
}

#[test]
fn test_boolean_or() {
    assert_eq!(Value::Boolean(true), execute("true or false"));
    assert_eq!(Value::Boolean(true), execute("true or true"));
    assert_eq!(Value::Boolean(false), execute("false or false"));
}

#[test]
fn test_boolean_not() {
    assert_eq!(Value::Boolean(false), execute("not true"));
    assert_eq!(Value::Boolean(true), execute("not false"));

    assert_eq!(Value::Boolean(true), execute("not false and true"));
    assert_eq!(Value::Boolean(false), execute("false or not true"));
}
