use crate::{encoder::encode, expressifier::expressify, parser::parse, tokenizer::Source};

pub mod encoder;
pub mod expressifier;
pub mod inst_encoder;
pub mod parser;
pub mod tokenizer;
pub mod utils;

pub fn compile(src: &str, path: &str) -> Result<String, utils::Error> {
	let mut source = Source::new(src, path);
	let mut lines = parse(&mut source)?;
	let bin = encode(&mut lines, &source)?;
	let res = expressify(&bin);
	Ok(res)
}
