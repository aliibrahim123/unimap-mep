use std::collections::HashMap;

use compact_str::CompactString;

use crate::{
	inst_encoder::encode_inst,
	parser::{AsmLine, AsmLineKind, Const, ConstKind, Operand, OperandKind, Reg, SImd, Shift},
	tokenizer::{Source, Span},
	utils::{Error, bit_insert, err},
};

fn reg_code(reg: Reg) -> u8 {
	#[rustfmt::skip]
	return match reg {
		Reg::R0  => 0,  Reg::R1  => 1,  Reg::R2  => 2,  Reg::R3  => 3,
		Reg::R4  => 4,  Reg::R5  => 5,  Reg::R6  => 6,  Reg::R7  => 7,
		Reg::R8  => 8,  Reg::R9  => 9,  Reg::R10 => 10, Reg::R11 => 11,
		Reg::R12 => 12, Reg::R13 => 13, Reg::R14 => 14, Reg::R15 => 15,
		Reg::R16 => 16, Reg::R17 => 17, Reg::R18 => 18, Reg::R19 => 19,
		Reg::R20 => 20, Reg::R21 => 21, Reg::R22 => 22, Reg::R23 => 23,
		Reg::R24 => 24, Reg::R25 => 25, Reg::R26 => 26, Reg::R27 => 27,
		Reg::R28 => 28, Reg::R29 => 29, Reg::R30 => 30, Reg::R31 => 31,
		Reg::PC  => 0,  Reg::C0  => 0,
	};
}

pub type LabelIndexes = HashMap<CompactString, u64>;

fn line_stats(line: &AsmLine) -> (u64, u64) {
	match &line.kind {
		AsmLineKind::Const(cons) => match &cons.kind {
			ConstKind::U8(_) | ConstKind::I8(_) => (1, 1),
			ConstKind::U16(_) | ConstKind::I16(_) => (2, 2),
			ConstKind::U32(_) | ConstKind::I32(_) => (4, 4),
			ConstKind::U64(_) | ConstKind::I64(_) => (8, 8),
			ConstKind::Bytes(bytes) => (bytes.len() as u64, 1),
			ConstKind::Str(str) => (str.len() as u64, 1),
		},
		AsmLineKind::Inst(_) => (4, 4),
	}
}

fn resolve_offsets(lines: &mut [AsmLine]) -> Result<(LabelIndexes, usize), Error> {
	let mut label_indexes = HashMap::new();
	let mut offset = 0u64;

	for line in lines {
		let (len, align) = line_stats(&line);
		if !offset.is_multiple_of(align) {
			let new_offset = offset.next_multiple_of(align);
			line.pad = new_offset - offset;
			offset = new_offset;
		}
		if let Some(label) = &line.label {
			label_indexes.insert(label.name.clone(), offset);
		}
		offset += len;
	}

	Ok((label_indexes, offset as usize))
}

fn encode_const(bin: &mut Vec<u8>, cons: &Const) {
	use ConstKind::*;
	match &cons.kind {
		U8(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		I8(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		U16(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		I16(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		U32(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		I32(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		U64(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		I64(nb) => bin.extend_from_slice(&nb.to_le_bytes()),
		Bytes(bytes) => bin.extend_from_slice(bytes),
		Str(str) => bin.extend_from_slice(str.as_bytes()),
	}
}

pub fn encode(lines: &mut [AsmLine], source: &Source) -> Result<Vec<u8>, Error> {
	let (label_indexes, len) = resolve_offsets(lines)?;
	let mut bin = Vec::with_capacity(len);

	for line in lines {
		if line.pad > 0 {
			bin.resize(bin.len() + line.pad as usize, 0);
		}
		match &line.kind {
			AsmLineKind::Const(cons) => encode_const(&mut bin, cons),
			AsmLineKind::Inst(inst) => {
				let inst = encode_inst(inst, bin.len() as u64, &label_indexes, source)?;
				bin.extend_from_slice(&inst.to_le_bytes());
			}
		}
	}

	if !bin.len().is_multiple_of(16) {
		bin.resize(bin.len().next_multiple_of(16), 0);
	}

	Ok(bin)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstBuilder<'a> {
	pub inst: u32,
	pub index: u8,
	pub source: &'a Source<'a>,
	pub label_indexes: &'a LabelIndexes,
	pub inst_index: u64,
}

impl InstBuilder<'_> {
	pub fn new<'b>(
		source: &'b Source<'b>, label_indexes: &'b LabelIndexes, inst_index: u64,
	) -> InstBuilder<'b> {
		InstBuilder { inst: 0, index: 0, source, label_indexes, inst_index }
	}
	pub fn finish(self) -> u32 {
		assert!(self.index == 32);
		self.inst
	}
	pub fn b(self, value: u32, len: u8) -> Self {
		Self {
			inst: bit_insert(self.inst, value, self.index, len),
			index: self.index + len,
			..self
		}
	}
	pub fn b1(self, value: u8) -> Self {
		self.b(value as u32, 1)
	}
	pub fn b2(self, value: u8) -> Self {
		self.b(value as u32, 2)
	}
	pub fn b3(self, value: u8) -> Self {
		self.b(value as u32, 3)
	}
	pub fn b4(self, value: u8) -> Self {
		self.b(value as u32, 4)
	}
	pub fn b5(self, value: u8) -> Self {
		self.b(value as u32, 5)
	}
	pub fn b6(self, value: u8) -> Self {
		self.b(value as u32, 6)
	}
	pub fn b8(self, value: u8) -> Self {
		self.b(value as u32, 8)
	}
	pub fn gpr(self, reg: Reg) -> Self {
		self.b(reg_code(reg) as u32, 5)
	}
	pub fn cond(self, reg: Reg, span: Span) -> Result<Self, Error> {
		match reg {
			Reg::C0 => Ok(self.b(0, 5)),
			Reg::R0 => err!("r0 can not be used as condition register", (span, self.source)),
			_ => Ok(self.b(reg_code(reg) as u32, 5)),
		}
	}
	pub fn gpr_pc(self, reg: Reg, span: Span) -> Result<Self, Error> {
		match reg {
			Reg::PC => Ok(self.b(0, 5)),
			Reg::R0 => err!("r0 can not be used here", (span, self.source)),
			_ => Ok(self.b(reg_code(reg) as u32, 5)),
		}
	}
	pub fn u_imd(self, value: u64, len: u8, span: Span) -> Result<Self, Error> {
		if value > (1 << len) - 1 {
			let src = self.source.source_of(span);
			return err!("number ({src}) out of range for u{len}_imd", (span, self.source));
		}
		Ok(self.b(value as u32, len))
	}
	pub fn s_imd(self, imd: &SImd, len: u8, span: Span) -> Result<Self, Error> {
		let value = match imd {
			SImd::I64(nb) => *nb,
			SImd::Label(label) => {
				let Some(ind) = self.label_indexes.get(&label.name) else {
					return err!("label {label} not found", (span, self.source));
				};
				*ind as i64 - self.inst_index as i64
			}
		};
		if value > (1 << (len - 1)) - 1 || value < -(1 << (len - 1)) {
			let what = if let SImd::I64(label) = imd {
				format!("label \"{label}\" offset")
			} else {
				let src = self.source.source_of(span);
				format!("number ({src})")
			};
			return err!("{what} out of range for s{len}_imd", (span, self.source));
		}
		Ok(self.b(value as u32, len))
	}
	pub fn shift(self, shift: Shift) -> Self {
		match shift {
			Shift::SHL(amount) => self.b(0, 2).b(amount as u32, 6),
			Shift::SHR(amount) => self.b(1, 2).b(amount as u32, 6),
			Shift::SAR(amount) => self.b(2, 2).b(amount as u32, 6),
			Shift::ROL(amount) => self.b(3, 2).b(amount as u32, 6),
		}
	}
	pub fn limd_l0(self, level: u8) -> Self {
		self.b1(if level == 64 { 0 } else { 1 })
	}
	pub fn logic_imd(self, level: u8, ones: u8, rot: u8) -> Self {
		let ones = ones - 1;
		let ones = match level {
			64 => ones,
			32 => ones,
			16 => ones | 0b100000,
			8 => ones | 0b110000,
			4 => ones | 0b111000,
			2 => ones | 0b111100,
			_ => unreachable!(),
		};
		self.b6(ones).b6(rot)
	}
}
