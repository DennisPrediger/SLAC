# SLAC: Simple Logic & Arithmetic Compiler

SLAC is a small compiler which converts a single expression statement into an AST. 

Written in Rust, it compiles easily as an executable, wasm module, or standalone DLL.
Delphi syntax was chosen for the expression syntax due to its simplicity.

# Example

```rust
use slac::{ast::Expression, compile};

fn main() {
    let ast = compile("1 * 2 + 3");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(1.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Star,
        }),
        right: Box::new(Expression::Literal(Token::Number(3.0))),
        operator: Token::Plus,
    };

    assert_eq!(result, Ok(expected));
}
```

# Installation

The minimum required Rust toolchain version for SLAC is **1.70.0** or higher. 

Use `cargo add` to install the library as a dependency in your application.

# License

Copyright 2023 Dennis Prediger

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.