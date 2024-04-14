use std::{env, process::ExitCode};

use slac::{optimize, Result, Value};

fn execute(source: &str) -> Result<Value> {
    let mut ast = slac::compile(&source)?;
    let mut env = slac::StaticEnvironment::default();
    slac::stdlib::extend_environment(&mut env);
    slac::check_variables_and_functions(&env, &ast)?;

    optimize(&mut ast)?;

    let result = slac::execute(&env, &ast)?;
    Ok(result)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if let Some(source) = args.get(1) {
        match execute(&source) {
            Ok(result) => {
                println!("{result}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                println!("Error: {error}");
                ExitCode::FAILURE
            }
        }
    } else {
        println!("Error: no script provided");
        ExitCode::FAILURE
    }
}
