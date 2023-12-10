#[cfg(feature = "serde")]
mod test {

    use minify::json::minify;
    use slac::{
        check_variables_and_functions, compile, stdlib::NativeResult, Expression, Operator,
        StaticEnvironment,
    };

    fn test_serialize(script: &str, expected: &str) {
        let ast = compile(script).unwrap();
        let json = serde_json::to_string(&ast).unwrap();
        assert_eq!(minify(expected), json);
    }

    fn test_roundtrip(script: &str) {
        let input = compile(script).unwrap();
        let json = serde_json::to_value(&input).unwrap();
        let output = serde_json::from_value::<Expression>(json).unwrap();

        assert_eq!(input, output);
    }

    fn dummy_func(_params: &[slac::Value]) -> NativeResult {
        Ok(slac::Value::Boolean(false))
    }

    fn test_validate(script: &str) {
        let input = compile(script).unwrap();
        let mut env = StaticEnvironment::default();
        env.add_function("max", dummy_func, Some(2), 0);
        env.add_function("some_func", dummy_func, Some(1), 0);
        env.add_variable("some_var", slac::Value::Boolean(false));

        assert!(check_variables_and_functions(&env, &input).is_ok());
    }

    fn test_json(script: &str, expected: &str) {
        test_validate(script);
        test_serialize(script, expected);
        test_roundtrip(script);
    }

    #[test]
    fn serialize_simple() {
        let expected = r#"
        {
          "type": "binary",
          "left": {
            "type": "literal",
            "value": 1.0
          },
          "right": {
            "type": "literal",
            "value": 2.0
          },
          "operator": "plus"
        }"#;

        test_json("1+ 2", expected);
    }

    #[test]
    fn serialize_function() {
        let expected = r#"
        {
          "type": "binary",
          "left": {
            "type": "call",
            "name": "max",
            "params": [
              {
                "type": "literal",
                "value": 10.0
              },
              {
                "type": "literal",
                "value": 20.0
              }
            ]
          },
          "right": {
            "type": "literal",
            "value": 5.0
          },
          "operator": "greater"
        }
        "#;

        test_json("max(10, 20) > 5", expected);
    }

    #[test]
    fn full_syntax() {
        let script = r#"true and not false and
                              10 + 20 - 30 < 50 * 5 / 25 and
                              10 div 3 <= 10 mod 3 or 
                              some_func(['hello', 1, true]) > some_var or 
                              7 >= 8 or  9 <> 10 and
                              'Apple' + 'Pen' = 'ApplePen'
                              "#;
        let expected = r#"
        {
          "type": "binary",
          "left": {
            "type": "binary",
            "left": {
              "type": "binary",
              "left": {
                "type": "binary",
                "left": {
                  "type": "binary",
                  "left": {
                    "type": "binary",
                    "left": {
                      "type": "literal",
                      "value": true
                    },
                    "right": {
                      "type": "unary",
                      "right": {
                        "type": "literal",
                        "value": false
                      },
                      "operator": "not"
                    },
                    "operator": "and"
                  },
                  "right": {
                    "type": "binary",
                    "left": {
                      "type": "binary",
                      "left": {
                        "type": "binary",
                        "left": {
                          "type": "literal",
                          "value": 10.0
                        },
                        "right": {
                          "type": "literal",
                          "value": 20.0
                        },
                        "operator": "plus"
                      },
                      "right": {
                        "type": "literal",
                        "value": 30.0
                      },
                      "operator": "minus"
                    },
                    "right": {
                      "type": "binary",
                      "left": {
                        "type": "binary",
                        "left": {
                          "type": "literal",
                          "value": 50.0
                        },
                        "right": {
                          "type": "literal",
                          "value": 5.0
                        },
                        "operator": "multiply"
                      },
                      "right": {
                        "type": "literal",
                        "value": 25.0
                      },
                      "operator": "divide"
                    },
                    "operator": "less"
                  },
                  "operator": "and"
                },
                "right": {
                  "type": "binary",
                  "left": {
                    "type": "binary",
                    "left": {
                      "type": "literal",
                      "value": 10.0
                    },
                    "right": {
                      "type": "literal",
                      "value": 3.0
                    },
                    "operator": "div"
                  },
                  "right": {
                    "type": "binary",
                    "left": {
                      "type": "literal",
                      "value": 10.0
                    },
                    "right": {
                      "type": "literal",
                      "value": 3.0
                    },
                    "operator": "mod"
                  },
                  "operator": "lessEqual"
                },
                "operator": "and"
              },
              "right": {
                "type": "binary",
                "left": {
                  "type": "call",
                  "name": "some_func",
                  "params": [
                    {
                      "type": "array",
                      "expressions": [
                        {
                          "type": "literal",
                          "value": "hello"
                        },
                        {
                          "type": "literal",
                          "value": 1.0
                        },
                        {
                          "type": "literal",
                          "value": true
                        }
                      ]
                    }
                  ]
                },
                "right": {
                  "type": "variable",
                  "name": "some_var"
                },
                "operator": "greater"
              },
              "operator": "or"
            },
            "right": {
              "type": "binary",
              "left": {
                "type": "literal",
                "value": 7.0
              },
              "right": {
                "type": "literal",
                "value": 8.0
              },
              "operator": "greaterEqual"
            },
            "operator": "or"
          },
          "right": {
            "type": "binary",
            "left": {
              "type": "binary",
              "left": {
                "type": "literal",
                "value": 9.0
              },
              "right": {
                "type": "literal",
                "value": 10.0
              },
              "operator": "notEqual"
            },
            "right": {
              "type": "binary",
              "left": {
                "type": "binary",
                "left": {
                  "type": "literal",
                  "value": "Apple"
                },
                "right": {
                  "type": "literal",
                  "value": "Pen"
                },
                "operator": "plus"
              },
              "right": {
                "type": "literal",
                "value": "ApplePen"
              },
              "operator": "equal"
            },
            "operator": "and"
          },
          "operator": "or"
        }"#;
        test_json(script, expected);
    }

    #[test]
    fn nested_json() {
        let expected = r#"
        {
          "type": "array",
          "expressions": [
            {
              "type": "array",
              "expressions": [
                {
                  "type": "literal",
                  "value": 1.0
                },
                {
                  "type": "literal",
                  "value": 2.0
                }
              ]
            },
            {
              "type": "literal",
              "value": 3.0
            }
          ]
        }"#;
        test_json("[[1, 2], 3]", expected)
    }

    #[test]
    fn zero_value() {
        let json = r#"{
          "type": "binary",
          "left": {
            "type": "literal",
            "value": 1
          },
          "right": {
            "type": "literal",
            "value": 0
          },
          "operator": "minus"
        }"#;

        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: slac::Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: slac::Value::Number(0.0),
            }),
            operator: Operator::Minus,
        };

        let ast = serde_json::from_str::<Expression>(json).unwrap();

        assert_eq!(expected, ast);
    }
}
