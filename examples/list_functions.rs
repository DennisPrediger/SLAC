use std::process::ExitCode;

use slac::stdlib::extend_environment;
use slac::StaticEnvironment;

fn main() -> ExitCode {
    let mut env = StaticEnvironment::default();
    extend_environment(&mut env);

    for func in env.list_functions() {
        println!("function {}{}", func.name, func.params);
    }

    ExitCode::SUCCESS
}
