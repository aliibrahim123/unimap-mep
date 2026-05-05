use std::{
	borrow::Cow,
	fmt::{Debug, Display},
};

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
	pub fn source_of(&self, span: Span) -> &str {
		&self.src[span.0 as usize..span.1 as usize]
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
		Err(0) => return (1, src[0..ind as usize].chars().count() as u32 + 1),
		Err(line) => line - 1,
	};
	let spaned = &src[line_poses[line] as usize..ind as usize];
	((line + 2) as u32, spaned.chars().count() as u32)
}
impl Span {
	pub fn none() -> Span {
		Span(0, 0)
	}
	pub fn point(ind: u32) -> Span {
		Span(ind, ind + 1)
	}
	pub fn pos(&self, source: &Source) -> Pos {
		Pos { start: pos_of(source, self.0), end: pos_of(source, self.1 - 1) }
	}
	pub fn is_none(&self) -> bool {
		self.0 == 0 && self.1 == 0
	}
	pub fn join(a: Span, b: Span) -> Span {
		Span(a.0, b.1)
	}
}
impl Debug for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Span({}..{})", self.0, self.1)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
	pub span: Span,
	pub kind: TokenKind<'a>,
}
impl Token<'_> {
	pub const EOF: Token<'static> = Token { span: Span(0, 0), kind: TokenKind::Eof };
	pub fn display<'a>(&'a self, source: &'a Source) -> &'a str {
		self.kind.display(self.span, source)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'a> {
	Ident(&'a str),
	Str(String),
	Nb(u64),

	Dot,
	Comma,
	Colon,
	Plus,
	Minus,
	Eq,

	ParanOpen,
	ParanClose,
	BracketOpen,
	BracketClose,

	NL,
	Eof,
}
impl TokenKind<'_> {
	pub fn display<'a>(&'a self, span: Span, source: &'a Source) -> &'a str {
		use TokenKind::*;
		match self {
			Ident(ident) => ident,
			Str(_) | Nb(_) => source.source_of(span),
			_ => self.symbol_fmt(),
		}
	}
	pub fn symbol_fmt(&self) -> &str {
		use TokenKind::*;
		match self {
			Ident(_) | Str(_) | Nb(_) => panic!(),
			NL => "\\n",
			Dot => ".",
			Comma => ",",
			Colon => ":",
			Plus => "+",
			Minus => "-",
			Eq => "=",
			ParanOpen => "(",
			ParanClose => ")",
			BracketOpen => "[",
			BracketClose => "]",
			Eof => "EOF",
		}
	}
}

pub fn unexpected_token<T>(
	token: impl Display, expected: &str, span: Span, source: &Source,
) -> Result<T, Error> {
	match expected {
		"" => err!("unexpected token ({token})", (span, source)),
		_ => err!("unexpected token ({token}), expected {expected}", (span, source)),
	}
}
pub fn end_of_input<T>(expected: &str, source: &Source) -> Result<T, Error> {
	match expected {
		"" => err!("unexpected end of input", (Span::none(), source)),
		_ => {
			err!("unexpected end of input, expected {expected}", (Span::none(), source))
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

fn resolve_escape_codes(str: &str, start_ind: u32, source: &Source) -> Result<String, Error> {
	let mut cur_ind = 0;
	let mut res = String::new();

	let invalid_escape_code = |start, end| {
		err!(
			"invalid escape code",
			(Span(start_ind + start as u32, start_ind + end as u32), source)
		)
	};

	while let Some(ind) = str.find_after('\\', cur_ind) {
		res.push_str(&str[cur_ind..ind]);
		cur_ind = ind + 2;
		res.push(match str.char_at(ind + 1).unwrap() {
			'n' => '\n',
			'r' => '\r',
			't' => '\t',
			'"' => '"',
			'\\' => '\\',
			'x' => {
				let Some(hex) = str.get(ind + 2..ind + 4) else {
					return invalid_escape_code(ind, str.len());
				};
				let Ok(code) = u8::from_str_radix(hex, 16) else {
					return invalid_escape_code(ind, ind + 4);
				};
				let Some(char) = char::from_u32(code as u32) else {
					return invalid_escape_code(ind, ind + 4);
				};
				cur_ind = ind + 4;
				char
			}
			'u' => {
				if str.char_at(ind + 2) != Some('{') {
					return invalid_escape_code(ind, ind + 3);
				}
				let Some(end) = str.find_after('}', ind + 3) else {
					return invalid_escape_code(ind, str.len());
				};
				let Ok(code) = u32::from_str_radix(&str[ind + 3..end], 16) else {
					return invalid_escape_code(ind, end);
				};
				let Some(char) = char::from_u32(code) else {
					return invalid_escape_code(ind, end);
				};
				cur_ind = end + 1;
				char
			}
			c => unexpected_token(c, "escape code", Span::point(start_ind + ind as u32), source)?,
		});
	}
	Ok(res)
}

fn count_lines(raw: &str, start_ind: u32, line_poses: &mut Vec<u32>) {
	for (ind, _) in raw.char_indices().filter(|(_, c)| *c == '\n') {
		line_poses.push(start_ind + ind as u32);
	}
}

pub fn tokenize<'a>(source: &mut Source<'a>) -> Result<Vec<Token<'a>>, Error> {
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
					span: Span(ind as u32, end as u32),
					kind: Ident(&src[ind..end]),
				});
				ind = end;
			}

			'0'..='9' => {
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

				let nb_raw = &src[if base == 10 { start } else { start + 2 }..end];
				let span = Span(start as u32, end as u32);
				let nb = strip_dashes_in_nb(nb_raw, start, source)?;
				let Ok(nb) = u64::from_str_radix(&nb, base) else {
					return err!("large number ({nb_raw})", (span, source));
				};
				tokens.push(Token { span, kind: Nb(nb) });
				ind = end;
			}

			'.' => push_single!(Dot),
			',' => push_single!(Comma),
			':' => push_single!(Colon),
			'+' => push_single!(Plus),
			'-' => push_single!(Minus),
			'=' => push_single!(Eq),

			'(' => push_single!(ParanOpen),
			')' => push_single!(ParanClose),
			'[' => push_single!(BracketOpen),
			']' => push_single!(BracketClose),

			'"' => {
				let start_ind = ind;
				let end = loop {
					let Some(end) = src.find_after('"', ind + 1) else {
						return end_of_input("\"", source);
					};
					if src.char_at(end - 1) == Some('\\') && src.char_at(end - 2) != Some('\\') {
						ind = end;
						continue;
					}
					break end;
				};

				count_lines(&src[start_ind..end], start_ind as u32, &mut source.line_poses);
				let str =
					resolve_escape_codes(&src[start_ind + 1..end], (start_ind + 1) as u32, source)?;
				tokens.push(Token { span: Span(start_ind as u32, end as u32 + 1), kind: Str(str) });
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
					count_lines(&src[ind..end], ind as u32, &mut source.line_poses);
					ind = end + 2;
				}
				_ => return unexpected_token(cur_char, "", Span::point(ind as u32), source),
			},

			_ => return unexpected_token(cur_char, "", Span::point(ind as u32), source),
		}
	}

	tokens.push(Token::EOF);
	Ok(tokens)
}
