use std::fmt::{Display, Write};

use crate::tokenizer::{Source, Span};

pub trait StrExt {
	fn char_at(&self, index: usize) -> Option<char>;
	fn find_after(&self, pat: char, index: usize) -> Option<usize>;
	fn find_after_str(&self, pat: &str, index: usize) -> Option<usize>;
	fn while_matching<F: Fn(char) -> bool>(&self, index: usize, f: F) -> usize;
}
impl StrExt for str {
	fn char_at(&self, index: usize) -> Option<char> {
		self[index..].chars().next()
	}
	fn find_after(&self, pat: char, index: usize) -> Option<usize> {
		self[index..].find(pat).map(|i| i + index)
	}
	fn find_after_str(&self, pat: &str, index: usize) -> Option<usize> {
		self[index..].find(pat).map(|i| i + index)
	}
	fn while_matching<F: Fn(char) -> bool>(&self, mut index: usize, fun: F) -> usize {
		for char in self[index..].chars() {
			if !fun(char) {
				break;
			}
			index += char.len_utf8();
		}
		index
	}
}

pub fn bit_insert(value: u32, src: u32, offset: u8, len: u8) -> u32 {
	let mask = (1 << len) - 1;
	(value & !(mask << offset)) | ((src & mask) << offset)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
	pub msg: Box<str>,
	pub span: Span,
}
impl std::error::Error for Error {}
impl Error {
	pub fn new(mut msg: String, span: Span, source: &Source) -> Self {
		write!(msg, "\n  --> {}", source.path).unwrap();

		if !span.is_none() {
			write!(msg, ":{}", span.pos(source)).unwrap()
		}
		Self { msg: msg.into_boxed_str(), span }
	}
}
impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}
macro_rules! err {
	($msg:expr, ($span:expr, $source:expr)) => {
		Err(crate::utils::Error::new(format!($msg), $span, $source))
	};
}
pub(crate) use err;
