use std::{
	cell::Cell,
	fmt::{Debug, Display},
};

use compact_str::CompactString;

use crate::{
	tokenizer::{Source, Span, Token, TokenKind, end_of_input, tokenize, unexpected_token},
	utils::{Error, err},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
	pub name: CompactString,
	pub span: Span,
}
impl Display for Ident {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstKind {
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),

	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),

	Bytes(Vec<u8>),
	Str(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Const {
	pub kind: ConstKind,
	pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
	Pos,
	Neg,
}
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
	R0,  R1,  R2,  R3,  R4,  R5,  R6,  R7,
	R8,  R9,  R10, R11, R12, R13, R14, R15,
	R16, R17, R18, R19, R20, R21, R22, R23,
	R24, R25, R26, R27, R28, R29, R30, R31,
	PC, C0
}
impl Reg {
	pub fn name(&self) -> &'static str {
		use Reg::*;
		#[rustfmt::skip]
		return match self {
			R0  => "r0",  R1  => "r1",  R2  => "r2",  R3  => "r3",
			R4  => "r4",  R5  => "r5",  R6  => "r6",  R7  => "r7",
			R8  => "r8",  R9  => "r9",  R10 => "r10", R11 => "r11", 
			R12 => "r12", R13 => "r13", R14 => "r14", R15 => "r15",
			R16 => "r16", R17 => "r17", R18 => "r18", R19 => "r19",
			R20 => "r20", R21 => "r21", R22 => "r22", R23 => "r23",
			R24 => "r24", R25 => "r25", R26 => "r26", R27 => "r27",
			R28 => "r28", R29 => "r29", R30 => "r30", R31 => "r31",
			PC  => "pc",  C0  => "c0",
		};
	}
}
impl Display for Reg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
	SHL(u8),
	SHR(u8),
	SAR(u8),
	ROL(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SImd {
	I64(i64),
	Label(Ident),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandKind {
	GPR(Reg),
	PC(Reg),
	C0(Reg),
	ShReg(Reg, Shift),
	UImd(u64),
	SImd(SImd),
	LogicImd { level: u8, ones: u8, rot: u8 },
	Offset(SImd),
	BaseOffset { base: Reg, offset: i64, offset_span: Span, writeback: bool },
	BaseIndex { base: Reg, index: Reg, shift: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operand {
	pub kind: OperandKind,
	pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inst {
	pub mnemonic: Ident,
	pub operands: Vec<Operand>,
	pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmLineKind {
	Const(Const),
	Inst(Inst),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmLine {
	pub label: Option<Ident>,
	pub pad: u64,
	pub kind: AsmLineKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
	offset: u64,
	span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
	pub lines: Vec<AsmLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cursor<'a> {
	pub tokens: &'a [Token<'a>],
	pub ind: Cell<usize>,
	pub source: &'a Source<'a>,
}

use TokenKind::*;
impl<'a> Cursor<'a> {
	pub fn new<'b>(tokens: &'b [Token<'b>], source: &'b Source) -> Cursor<'b> {
		Cursor { tokens, ind: Cell::new(0), source }
	}
	pub fn is_end(&self) -> bool {
		self.ind.get() >= self.tokens.len() - 1
	}
	pub fn last(&self) -> &Token<'a> {
		debug_assert!(self.ind.get() > 0);
		&self.tokens[self.ind.get() - 1]
	}
	pub fn peek(&self) -> &Token<'a> {
		&self.tokens[self.ind.get()]
	}
	pub fn peek_next(&self) -> &Token<'a> {
		&self.tokens.get(self.ind.get() + 1).unwrap_or(&Token::EOF)
	}
	pub fn skip(&self) -> Span {
		let span = self.peek().span;
		self.ind.set(self.ind.get() + 1);
		span
	}
	pub fn test(&self, kind: TokenKind) -> bool {
		self.peek().kind == kind
	}
	pub fn consume(&self, kind: TokenKind) -> Result<Span, Error> {
		if self.is_end() {
			return end_of_input(&format!("({})", kind.symbol_fmt()), self.source);
		}
		let token = self.peek();
		if token.kind != kind {
			let expected = &format!("({})", kind.symbol_fmt());
			return unexpected_token(token.display(self.source), expected, token.span, self.source);
		}
		self.skip();
		Ok(token.span)
	}
	pub fn try_eat(&self, kind: TokenKind<'a>) -> bool {
		if self.peek().kind != kind {
			return false;
		}
		self.skip();
		true
	}
	pub fn try_consume(&self, kind: TokenKind) -> Option<Span> {
		let token = self.peek();
		(token.kind == kind).then(|| {
			self.skip();
			token.span
		})
	}
	pub fn consume_ident(&self) -> Result<Ident, Error> {
		let token = self.peek();
		let Ident(ident) = token.kind else {
			return self.err_expected("identifier");
		};
		self.skip();
		Ok(Ident { name: CompactString::from(ident), span: token.span })
	}
	pub fn try_consume_ident(&self) -> Option<Ident> {
		let token = self.peek();
		if let Ident(ident) = token.kind {
			self.skip();
			Some(Ident { name: CompactString::from(ident), span: token.span })
		} else {
			None
		}
	}
	pub fn consume_nb(&self) -> Result<(u64, Span), Error> {
		let token = self.peek();
		let Nb(nb) = token.kind else {
			return self.err_expected("number");
		};
		self.skip();
		Ok((nb, token.span))
	}
	pub fn try_consume_nb(&self) -> Option<(u64, Span)> {
		let token = self.peek();
		if let Nb(nb) = token.kind {
			self.skip();
			Some((nb, token.span))
		} else {
			None
		}
	}
	pub fn consume_str(&self) -> Result<(String, Span), Error> {
		let token = self.peek();
		let Str(str) = &token.kind else {
			return self.err_expected("string");
		};
		self.skip();
		Ok((str.to_string(), token.span))
	}
	pub fn err_expected<T>(&self, expected: &str) -> Result<T, Error> {
		if self.is_end() {
			return end_of_input(expected, self.source);
		}
		unexpected_token(self.peek().display(self.source), expected, self.peek().span, self.source)
	}
}

fn check_nb_range(cur: &Cursor, nb_type: &str, nb: u64, max: u64, span: Span) -> Result<(), Error> {
	if nb > max { nb_out_of_range(cur, nb_type, span) } else { Ok(()) }
}
fn nb_out_of_range<T>(cur: &Cursor, nb_type: &str, span: Span) -> Result<T, Error> {
	let src = cur.source.source_of(span);
	err!("number ({src}) out of range for {nb_type}", (span, cur.source))
}
fn into_i64(cur: &Cursor, nb_type: &str, nb: u64, is_neg: bool, span: Span) -> Result<i64, Error> {
	if if is_neg { nb <= i64::MIN.unsigned_abs() } else { nb <= i64::MAX as u64 } {
		Ok(if is_neg { (nb as i64).wrapping_neg() } else { nb as i64 })
	} else {
		nb_out_of_range(cur, nb_type, span)
	}
}

fn try_parse_const(cur: &Cursor) -> Result<Option<Const>, Error> {
	let start_span = cur.peek().span;
	let Ident(ident) = cur.peek().kind else {
		return Ok(None);
	};

	fn match_u_nb<T: TryFrom<u64>>(
		cur: &Cursor, into: impl Fn(T) -> ConstKind,
	) -> Result<Option<Const>, Error> {
		let Token { span: start_span, kind: Ident(ident) } = cur.peek() else { unreachable!() };
		cur.skip();
		let (nb, nb_span) = cur.consume_nb()?;
		if let Ok(nb) = T::try_from(nb) {
			Ok(Some(Const { kind: into(nb), span: Span::join(*start_span, nb_span) }))
		} else {
			nb_out_of_range(cur, &format!("{ident} constant"), nb_span)
		}
	}
	fn match_i_nb<T: TryFrom<i64>>(
		cur: &Cursor, into: impl Fn(T) -> ConstKind,
	) -> Result<Option<Const>, Error> {
		let Token { span: start_span, kind: Ident(ident) } = cur.peek() else { unreachable!() };
		cur.skip();

		let sign_span = cur.peek().span;
		let is_neg = cur.try_eat(Minus);
		if !is_neg {
			cur.try_eat(Plus);
		}

		let (nb, nb_span) = cur.consume_nb()?;
		let nb_span = Span::join(sign_span, nb_span);
		let nb = into_i64(cur, &format!("{ident} constant"), nb, is_neg, nb_span)?;
		if let Ok(nb) = T::try_from(nb) {
			Ok(Some(Const { kind: into(nb), span: Span::join(*start_span, nb_span) }))
		} else {
			nb_out_of_range(cur, &format!("{ident} constant"), nb_span)
		}
	}

	match ident {
		"u8" => match_u_nb(cur, ConstKind::U8),
		"u16" => match_u_nb(cur, ConstKind::U16),
		"u32" => match_u_nb(cur, ConstKind::U32),
		"u64" => match_u_nb(cur, ConstKind::U64),
		"i8" => match_i_nb(cur, ConstKind::I8),
		"i16" => match_i_nb(cur, ConstKind::I16),
		"i32" => match_i_nb(cur, ConstKind::I32),
		"i64" => match_i_nb(cur, ConstKind::I64),
		"bytes" => {
			cur.skip();
			let mut bytes = Vec::new();
			loop {
				eat_newlines(cur);
				let (byte, span) = cur.consume_nb()?;
				let Ok(byte) = u8::try_from(byte) else {
					return nb_out_of_range(cur, "byte", span);
				};
				bytes.push(byte);
				if !cur.try_eat(Comma) {
					break;
				}
			}

			let span = Span::join(start_span, cur.last().span);
			Ok(Some(Const { kind: ConstKind::Bytes(bytes), span }))
		}
		"str" => {
			cur.skip();
			let (str, span) = cur.consume_str()?;
			let span = Span::join(start_span, span);
			Ok(Some(Const { kind: ConstKind::Str(str.clone()), span }))
		}
		_ => Ok(None),
	}
}

fn try_parse_gpr(ident: &str) -> Option<Reg> {
	match ident {
		"r0" => Some(Reg::R0),
		"r1" => Some(Reg::R1),
		"r2" => Some(Reg::R2),
		"r3" => Some(Reg::R3),
		"r4" => Some(Reg::R4),
		"r5" => Some(Reg::R5),
		"r6" => Some(Reg::R6),
		"r7" => Some(Reg::R7),
		"r8" => Some(Reg::R8),
		"r9" => Some(Reg::R9),
		"r10" => Some(Reg::R10),
		"r11" => Some(Reg::R11),
		"r12" => Some(Reg::R12),
		"r13" => Some(Reg::R13),
		"r14" => Some(Reg::R14),
		"r15" => Some(Reg::R15),
		"r16" => Some(Reg::R16),
		"r17" => Some(Reg::R17),
		"r18" => Some(Reg::R18),
		"r19" => Some(Reg::R19),
		"r20" => Some(Reg::R20),
		"r21" => Some(Reg::R21),
		"r22" => Some(Reg::R22),
		"r23" => Some(Reg::R23),
		"r24" => Some(Reg::R24),
		"r25" => Some(Reg::R25),
		"r26" => Some(Reg::R26),
		"r27" => Some(Reg::R27),
		"r28" => Some(Reg::R28),
		"r29" => Some(Reg::R29),
		"r30" => Some(Reg::R30),
		"r31" => Some(Reg::R31),
		_ => None,
	}
}

fn try_parse_simd(cur: &Cursor) -> Result<Option<(SImd, Span)>, Error> {
	if !matches!(cur.peek().kind, Minus | Plus) {
		return Ok(None);
	}
	let is_neg = cur.peek().kind == Minus;
	let sign_span = cur.skip();
	let (nb, nb_span) = cur.consume_nb()?;
	let nb = into_i64(cur, "signed immediate", nb, is_neg, nb_span)?;
	Ok(Some((SImd::I64(nb), Span::join(sign_span, nb_span))))
}
fn parse_logic_imd(cur: &Cursor, ident: &Ident) -> Result<Operand, Error> {
	cur.consume(ParanOpen)?;
	let mut level = 64;
	let mut level_span = Span::none();

	let (mut ones, mut ones_span) = cur.consume_nb()?;
	cur.consume(Comma)?;
	let (mut rot, mut rot_span) = cur.consume_nb()?;

	if cur.try_eat(Comma) {
		level = ones;
		level_span = ones_span;
		ones = rot;
		ones_span = rot_span;
		(rot, rot_span) = cur.consume_nb()?;
	}
	let end_span = cur.consume(ParanClose)?;

	if !matches!(level, 2 | 4 | 8 | 16 | 32 | 64) {
		let src = cur.source.source_of(level_span);
		return err!("invalid logic immediate level ({src})", (level_span, cur.source));
	}
	if ones == 0 || ones > level {
		let src = cur.source.source_of(ones_span);
		return err!("invalid logic immediate one_len ({src})", (ones_span, cur.source));
	}
	if rot >= level {
		let src = cur.source.source_of(rot_span);
		return err!("invalid logic immediate rot ({src})", (rot_span, cur.source));
	}

	let kind = OperandKind::LogicImd { level: level as u8, ones: ones as u8, rot: rot as u8 };
	return Ok(Operand { kind, span: Span::join(ident.span, end_span) });
}
fn parse_address(cur: &Cursor) -> Result<OperandKind, Error> {
	use OperandKind::{BaseIndex, BaseOffset, Offset};
	if let Some((nb, _)) = try_parse_simd(cur)? {
		return Ok(Offset(nb));
	}

	let ident = cur.consume_ident()?;
	let Some(base) = try_parse_gpr(&ident.name) else {
		return Ok(Offset(SImd::Label(ident)));
	};
	if !matches!(cur.peek().kind, Plus | Minus) {
		return Ok(BaseIndex { base, index: Reg::R0, shift: 0 });
	}

	let is_neg = cur.peek().kind == Minus;
	let sign_span = cur.skip();
	if let Some(ident) = cur.try_consume_ident() {
		if is_neg {
			return unexpected_token("(-)", "(+)", sign_span, cur.source);
		}
		let Some(index) = try_parse_gpr(&ident.name) else {
			return unexpected_token(&ident.name, "register", ident.span, cur.source);
		};

		let shift = if let Some(ident) = cur.try_consume_ident() {
			if ident.name != "shl" {
				return unexpected_token(&ident.name, "\"shl\"", ident.span, cur.source);
			}
			let (amount, amount_span) = cur.consume_nb()?;
			check_nb_range(cur, "shift amount", amount, 63, amount_span)?;
			amount as u8
		} else {
			0
		};

		Ok(BaseIndex { base, index, shift })
	} else {
		let writeback = cur.try_eat(Eq);
		let (offset, offset_span) = cur.consume_nb()?;
		let offset = into_i64(cur, "offset", offset, is_neg, sign_span)?;
		Ok(BaseOffset { base, offset, offset_span, writeback })
	}
}
fn parse_operand(cur: &Cursor) -> Result<Operand, Error> {
	use OperandKind::*;
	if let Some(ident) = cur.try_consume_ident() {
		if ident.name == "logic_imd" {
			return parse_logic_imd(cur, &ident);
		}
		if ident.name == "c0" {
			return Ok(Operand { span: ident.span, kind: C0(Reg::C0) });
		}
		if ident.name == "pc" {
			return Ok(Operand { span: ident.span, kind: PC(Reg::PC) });
		}
		let Some(reg) = try_parse_gpr(&ident.name) else {
			return Ok(Operand { span: ident.span, kind: SImd(self::SImd::Label(ident)) });
		};

		if let Some(sh_ident) = cur.try_consume_ident() {
			let (amount, amount_span) = cur.consume_nb()?;
			check_nb_range(cur, "shift amount", amount, 63, amount_span)?;

			let shift = match sh_ident.name.as_str() {
				"shl" => Shift::SHL(amount as u8),
				"shr" => Shift::SHR(amount as u8),
				"sar" => Shift::SAR(amount as u8),
				"rol" => Shift::ROL(amount as u8),
				_ => {
					let shift_op = "shift operator";
					return unexpected_token(&sh_ident.name, shift_op, sh_ident.span, cur.source);
				}
			};

			let span = Span::join(ident.span, amount_span);
			Ok(Operand { span, kind: ShReg(reg, shift) })
		} else {
			Ok(Operand { span: ident.span, kind: GPR(reg) })
		}
	} else if let Some((nb, span)) = cur.try_consume_nb() {
		Ok(Operand { span, kind: UImd(nb) })
	} else if let Some((nb, span)) = try_parse_simd(cur)? {
		Ok(Operand { span, kind: SImd(nb) })
	} else if let Some(start_span) = cur.try_consume(BracketOpen) {
		let kind = parse_address(cur)?;
		let end_span = cur.consume(BracketClose)?;
		Ok(Operand { span: Span::join(start_span, end_span), kind })
	} else {
		cur.err_expected("operand")
	}
}

fn parse_inst(cur: &Cursor) -> Result<Inst, Error> {
	let mut mnemonic = cur.consume_ident()?;
	while cur.try_eat(Dot) {
		mnemonic.name.push('.');
		if let src = cur.source.source_of(cur.peek().span)
			&& matches!(src, "8" | "16" | "32")
		{
			mnemonic.name.push_str(&src);
			mnemonic.span = Span::join(mnemonic.span, cur.skip());
		} else {
			let ident = cur.consume_ident()?;
			mnemonic.name.push_str(&ident.name);
			mnemonic.span = Span::join(mnemonic.span, ident.span);
		}
	}
	let start_span = mnemonic.span;

	if cur.test(NL) {
		return Ok(Inst { mnemonic, operands: vec![], span: start_span });
	}

	let mut operands = vec![parse_operand(cur)?];
	while cur.try_eat(Comma) {
		operands.push(parse_operand(cur)?);
	}
	let span = Span::join(start_span, operands.last().unwrap().span);
	Ok(Inst { mnemonic, operands, span })
}

fn eat_newlines(cur: &Cursor) {
	while cur.try_eat(NL) {}
}

pub fn parse(source: &mut Source) -> Result<Vec<AsmLine>, Error> {
	let tokens = tokenize(source)?;
	let cur = &Cursor::new(&tokens, source);
	eat_newlines(cur);

	let mut lines = vec![];
	while !cur.is_end() {
		let mut label = None;
		if cur.peek_next().kind == Colon {
			label = Some(cur.consume_ident()?);
			cur.consume(Colon)?;
			eat_newlines(cur);
		}
		let kind = if let Some(cons) = try_parse_const(cur)? {
			AsmLineKind::Const(cons)
		} else {
			AsmLineKind::Inst(parse_inst(cur)?)
		};
		lines.push(AsmLine { label, kind, pad: 0 });
		if !cur.is_end() {
			cur.consume(NL)?;
			eat_newlines(cur);
		}
	}

	Ok(lines)
}
