// The README is not part of the doc tests, so test the examples in here.

mod usage {
    use slac::{compile, Expression, Operator, Value};

    #[test]
    fn test_usage() {
        let ast = compile("1 * 2 + 3");

        let expected = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal {
                    value: Value::Number(1.0),
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Number(2.0),
                }),
                operator: Operator::Multiply,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(3.0),
            }),
            operator: Operator::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }
}

mod interpreter {
    use slac::{compile, execute, stdlib::add_stdlib, StaticEnvironment, Value};

    #[test]
    fn test_interpreter() {
        let ast = compile("max(some_var, 3) > 5").unwrap();
        let mut env = StaticEnvironment::default();
        add_stdlib(&mut env);
        env.add_var("some_var", Value::Number(42.0));

        let result = execute(&env, &ast);

        assert_eq!(result, Some(Value::Boolean(true)));
    }
}

#[cfg(feature = "serde")]
mod serialisation {
    use slac::{compile, execute, Expression, StaticEnvironment, Value};

    #[test]
    fn serialisation() {
        let input = compile("50 * 3 > 149").unwrap();
        let json = serde_json::to_value(&input).unwrap();

        // > Store the JSON in a database and load it on the client

        let output = serde_json::from_value::<Expression>(json).unwrap();
        let env = StaticEnvironment::default();

        let result = execute(&env, &output);

        assert_eq!(input, output);
        assert_eq!(result, Some(Value::Boolean(true)));
    }
}
