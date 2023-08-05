#[cfg(feature = "serde")]
mod test {

    use minify::json::minify;
    use slac::{ast::Expression, compile};

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

    fn test_json(script: &str, expected: &str) {
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
          },
          "right": {
            "type": "literal",
            "value": {
              "number": 5.0
            }
          },
          "operator": ">"
        }
        "#;

        test_json("max(10, 20) > 5", expected);
    }

    #[test]
    fn full_syntax() {
        let script = r#"true and not false and
                              10 + 20 - 30 < 50 * 5 / 25 and
                              10 div 3 <= 10 mod 3 or 
                              some_func(['array', 1, true]) > some_var or 
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
                      "value": {
                        "boolean": true
                      }
                    },
                    "right": {
                      "type": "unary",
                      "right": {
                        "type": "literal",
                        "value": {
                          "boolean": false
                        }
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
                          "value": {
                            "number": 10.0
                          }
                        },
                        "right": {
                          "type": "literal",
                          "value": {
                            "number": 20.0
                          }
                        },
                        "operator": "+"
                      },
                      "right": {
                        "type": "literal",
                        "value": {
                          "number": 30.0
                        }
                      },
                      "operator": "-"
                    },
                    "right": {
                      "type": "binary",
                      "left": {
                        "type": "binary",
                        "left": {
                          "type": "literal",
                          "value": {
                            "number": 50.0
                          }
                        },
                        "right": {
                          "type": "literal",
                          "value": {
                            "number": 5.0
                          }
                        },
                        "operator": "*"
                      },
                      "right": {
                        "type": "literal",
                        "value": {
                          "number": 25.0
                        }
                      },
                      "operator": "/"
                    },
                    "operator": "<"
                  },
                  "operator": "and"
                },
                "right": {
                  "type": "binary",
                  "left": {
                    "type": "binary",
                    "left": {
                      "type": "literal",
                      "value": {
                        "number": 10.0
                      }
                    },
                    "right": {
                      "type": "literal",
                      "value": {
                        "number": 3.0
                      }
                    },
                    "operator": "div"
                  },
                  "right": {
                    "type": "binary",
                    "left": {
                      "type": "literal",
                      "value": {
                        "number": 10.0
                      }
                    },
                    "right": {
                      "type": "literal",
                      "value": {
                        "number": 3.0
                      }
                    },
                    "operator": "mod"
                  },
                  "operator": "<="
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
                          "value": {
                            "string": "array"
                          }
                        },
                        {
                          "type": "literal",
                          "value": {
                            "number": 1.0
                          }
                        },
                        {
                          "type": "literal",
                          "value": {
                            "boolean": true
                          }
                        }
                      ]
                    }
                  ]
                },
                "right": {
                  "type": "variable",
                  "name": "some_var"
                },
                "operator": ">"
              },
              "operator": "or"
            },
            "right": {
              "type": "binary",
              "left": {
                "type": "literal",
                "value": {
                  "number": 7.0
                }
              },
              "right": {
                "type": "literal",
                "value": {
                  "number": 8.0
                }
              },
              "operator": ">="
            },
            "operator": "or"
          },
          "right": {
            "type": "binary",
            "left": {
              "type": "binary",
              "left": {
                "type": "literal",
                "value": {
                  "number": 9.0
                }
              },
              "right": {
                "type": "literal",
                "value": {
                  "number": 10.0
                }
              },
              "operator": "<>"
            },
            "right": {
              "type": "binary",
              "left": {
                "type": "binary",
                "left": {
                  "type": "literal",
                  "value": {
                    "string": "Apple"
                  }
                },
                "right": {
                  "type": "literal",
                  "value": {
                    "string": "Pen"
                  }
                },
                "operator": "+"
              },
              "right": {
                "type": "literal",
                "value": {
                  "string": "ApplePen"
                }
              },
              "operator": "="
            },
            "operator": "and"
          },
          "operator": "or"
        }"#;
        test_json(script, expected);
    }
}
