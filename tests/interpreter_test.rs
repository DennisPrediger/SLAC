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
fn add_number() {
    assert_eq!(Value::Number(100.0), execute("99 + 1"));
    assert_eq!(Value::Number(100.5), execute("99.2 + 1.3"));
}

#[test]
fn add_string() {
    assert_eq!(
        Value::String("Hello World".to_string()),
        execute("'Hello' + ' ' + 'World'")
    );
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
    assert_eq!(Value::Boolean(true), execute("true or false"));
    assert_eq!(Value::Boolean(true), execute("true or true"));
    assert_eq!(Value::Boolean(false), execute("false or false"));
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
}
