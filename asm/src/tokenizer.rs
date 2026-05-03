use std::{borrow::Cow, fmt::Display};

use crate::utils::{Error, StrExt, err};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source<'a> {
	pub src: &'a str,
	pub path: &'a str,
	pub line_poses: Vec<u32>,
}
impl Source<'_> {
	pub fn new<'a>(src: &'a str, path: &'a str) -> Source<'a> {
		Source { src, path, line_poses: Vec::new() }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(u32, u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
	start: (u32, u32),
	end: (u32, u32),
}
impl Display for Pos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Self { start, end } = self;
		if start == end {
			write!(f, "{}:{}", start.0, start.1)
		} else {
			write!(f, "{}:{}-{}:{}", self.start.0, self.start.1, self.end.0, self.end.1)
		}
	}
}

fn pos_of(source: &Source, ind: u32) -> (u32, u32) {
	let Source { src, line_poses, .. } = source;
	let line = match line_poses.binary_search(&ind) {
		Ok(line) => line,
		Err(line) => line - 1,
	};
	let spaned = &src[line_poses[line] as usize..ind as usize];
	((line + 1) as u32, spaned.chars().count() as u32)
}
impl Span {
	pub fn none() -> Span {
		Span(0, 0)
	}
	pub fn point(ind: u32) -> Span {
		Span(ind, ind)
	}
	pub fn pos(&self, source: &Source) -> Pos {
		Pos { start: pos_of(source, self.0), end: pos_of(source, self.1) }
	}
	pub fn is_none(&self) -> bool {
		self.0 == 0 && self.1 == 0
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
	pub span: Span,
	pub kind: TokenKind<'a>,
}
impl Display for Token<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.kind)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'a> {
	Ident(&'a str),
	Str(&'a str),
	Nb(u64),

	NL,
	Dot,
	Comma,
	Plus,
	Minus,
	Eq,

	ParanOpen,
	ParanClose,
	BracketOpen,
	BracketClose,
}
impl Display for TokenKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TokenKind::Ident(ident) => write!(f, "{ident}"),
			TokenKind::Str(str) => write!(f, "{str}"),
			TokenKind::Nb(nb) => write!(f, "{nb}"),
			TokenKind::NL => write!(f, "\\n"),
			TokenKind::Dot => write!(f, "."),
			TokenKind::Comma => write!(f, ","),
			TokenKind::Plus => write!(f, "+"),
			TokenKind::Minus => write!(f, "-"),
			TokenKind::Eq => write!(f, "="),
			TokenKind::ParanOpen => write!(f, "("),
			TokenKind::ParanClose => write!(f, ")"),
			TokenKind::BracketOpen => write!(f, "["),
			TokenKind::BracketClose => write!(f, "]"),
		}
	}
}

pub fn unexpected_token<T>(
	token: impl Display, expected: &str, span: Span, source: &Source,
) -> Result<T, Error> {
	match expected {
		"" => err!("parse error: unexpected token ({token})", (span, source)),
		_ => err!("parse error: unexpected token ({token}), expected {expected}", (span, source)),
	}
}
pub fn end_of_input<T>(expected: &str, source: &Source) -> Result<T, Error> {
	match expected {
		"" => err!("parse error: unexpected end of input", (Span::none(), source)),
		_ => {
			err!(
				"parse error: unexpected end of input, expected {expected}",
				(Span::none(), source)
			)
		}
	}
}

fn strip_dashes_in_nb<'a>(
	nb: &'a str, start_ind: usize, source: &'a Source<'a>,
) -> Result<Cow<'a, str>, Error> {
	let mut cur_ind = 0;
	let mut has_dash = false;
	while let Some(ind) = nb.find_after('_', cur_ind) {
		has_dash = true;
		if matches!(nb.char_at(ind - 1), None | Some('_')) {
			return unexpected_token('_', "", Span::point((start_ind + ind) as u32), source);
		}
		if matches!(nb.char_at(ind + 1), None | Some('_')) {
			return unexpected_token('_', "", Span::point((start_ind + ind) as u32), source);
		}
		cur_ind = ind + 1;
	}
	Ok(if has_dash { Cow::from(nb.replace('_', "")) } else { Cow::from(nb) })
}

pub fn tokenize<'a>(source: &'a mut Source<'a>) -> Result<Vec<Token<'a>>, Error> {
	use TokenKind::*;
	let src = &source.src;
	let mut tokens = Vec::new();
	let mut ind = 0usize;

	macro_rules! push_single {
		($token:expr) => {{
			tokens.push(Token { span: Span::point(ind as u32), kind: $token });
			ind += 1;
		}};
	}

	while let Some(cur_char) = src.char_at(ind) {
		match cur_char {
			' ' | '\t' | '\r' => ind += 1,
			'\n' => {
				source.line_poses.push(ind as u32);
				push_single!(NL);
			}

			'a'..='z' | 'A'..='Z' | '_' => {
				let end = src
					.while_matching(ind, |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));
				tokens.push(Token {
					span: Span(ind as u32, end as u32 - 1),
					kind: Ident(&src[ind..end]),
				});
				ind = end;
			}

			'0'..'9' => {
				let start = ind;
				let (base, end) = match (src.char_at(ind), src.char_at(ind + 1)) {
					(Some('0'), Some('b')) => {
						(2, src.while_matching(ind + 2, |c| matches!(c, '0' | '1' | '_')))
					}
					(Some('0'), Some('x')) => {
						let is_hex = |c| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F' | '_');
						(16, src.while_matching(ind + 2, is_hex))
					}
					_ => (10, src.while_matching(ind, |c| matches!(c, '0'..='9' | '_'))),
				};

				let nb_raw = &src[start..end];
				let span = Span(start as u32, end as u32 - 1);
				let nb = strip_dashes_in_nb(nb_raw, start, source)?;
				let Ok(nb) = u64::from_str_radix(&nb, base) else {
					return err!("parse error: large number ({nb_raw})", (span, source));
				};

				tokens.push(Token { span, kind: Nb(nb) });
				ind = end;
			}

			'.' => push_single!(Dot),
			',' => push_single!(Comma),
			'+' => push_single!(Plus),
			'-' => push_single!(Minus),
			'=' => push_single!(Eq),

			'(' => push_single!(ParanOpen),
			')' => push_single!(ParanClose),
			'[' => push_single!(BracketOpen),
			']' => push_single!(BracketClose),

			'"' => {
				let end = loop {
					let Some(end) = src.find_after('"', ind + 1) else {
						return end_of_input("\"", source);
					};
					if src.char_at(end - 1) == Some('\\') {
						ind = end;
						continue;
					}
					break end;
				};

				tokens.push(Token {
					span: Span(ind as u32, end as u32),
					kind: Str(&src[ind + 1..end]),
				});
				ind = end + 1;
			}

			'/' => match src.char_at(ind + 1) {
				Some('/') => {
					ind = src.find_after('\n', ind + 2).unwrap_or(src.len());
				}
				Some('*') => {
					let Some(end) = src.find_after_str("*/", ind + 2) else {
						return end_of_input("*/", source);
					};
					ind = end + 2;
				}
				_ => return unexpected_token(cur_char, "", Span::point(ind as u32), source),
			},

			_ => return unexpected_token(cur_char, "", Span::point(ind as u32), source),
		}
	}

	Ok(tokens)
}
