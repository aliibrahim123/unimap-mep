use std::fs::{canonicalize, read_to_string, write};

use clap::Parser;
use mep_asm::compile;

/// compile a mep assembly file
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(author, version, about = "a tool to compile mep assembly", long_about = None)]
struct Args {
	/// the assembly file to compile
	#[arg(value_name = "FILE")]
	input: String,
	/// the unimap file to output
	#[arg()]
	output: String,
}

fn main() {
	if let Err(err) = mainer() {
		eprintln!("{err}");
	}
}
fn mainer() -> Result<(), String> {
	let Args { input, output } = Args::parse();

	let input_path = canonicalize(&input).map_err(|_| format!("failed to read {input}"))?;
	let input = input_path.to_string_lossy();
	let src = read_to_string(&input_path).map_err(|_| format!("failed to read {input}"))?;

	let res = compile(&src, &input).map_err(|err| err.msg)?;

	write(&output, res).map_err(|_| format!("failed to write {output}"))?;

	Ok(())
}
