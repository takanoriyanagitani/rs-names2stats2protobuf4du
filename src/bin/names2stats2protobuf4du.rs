use std::io;
use std::process::ExitCode;

use rs_names2stats2protobuf4du::stdin2names2dtos2protos2stdout;

fn sub() -> Result<(), io::Error> {
    stdin2names2dtos2protos2stdout()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
