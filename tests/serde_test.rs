#[cfg(feature = "serde")]
mod test {

    use minify::json::minify;
    use slac::{ast::Expression, compile};

    fn test_serialize(script: &str, expected: &str) {
        let ast = compile(script).unwrap();
        let json = serde_json::to_string(&ast).unwrap();
        assert_eq!(minify(expected), json);
    }

    #[test]
    fn serialize_simple() {
        let expected = r#"
    {
      "type": "binary",
      "value": {
        "left": {
          "type": "literal",
          "value": {
            "number": 1.0
          }
        },
        "right": {
          "type": "literal",
          "value": {
            "number": 2.0
          }
        },
        "operator": "+"
      }
    }"#;

        test_serialize("1+ 2", expected);
    }

    #[test]
    fn serialize_function() {
        let expected = r#"
        {
          "type": "binary",
          "value": {
            "left": {
              "type": "call",
              "value": {
                "name": "max",
                "params": [
                  {
                    "type": "literal",
                    "value": {
                      "number": 10.0
                    }
                  },
                  {
                    "type": "literal",
                    "value": {
                      "number": 20.0
                    }
                  }
                ]
              }
            },
            "right": {
              "type": "literal",
              "value": {
                "number": 5.0
              }
            },
            "operator": ">"
          }
        }
        "#;

        test_serialize("max(10, 20) > 5", expected);
    }

    #[test]
    fn ser_de_round_trip() {
        let input = compile("1 + 2").unwrap();
        let json = serde_json::to_value(&input).unwrap();
        let output = serde_json::from_value::<Expression>(json).unwrap();

        assert_eq!(input, output);
    }
}
