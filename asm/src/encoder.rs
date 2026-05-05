use std::collections::HashMap;

use compact_str::CompactString;

use crate::{
	inst_encoder::encode_inst,
	parser::{AsmLine, AsmLineKind, Const, ConstKind, GPR, Operand, OperandKind},
	tokenizer::{Source, Span},
	utils::{Error, bit_insert, err},
};

fn reg_code(reg: GPR) -> u8 {
	#[rustfmt::skip]
	return match reg {
		GPR::R0  => 0,  GPR::R1  => 1,  GPR::R2  => 2,  GPR::R3  => 3,
		GPR::R4  => 4,  GPR::R5  => 5,  GPR::R6  => 6,  GPR::R7  => 7,
		GPR::R8  => 8,  GPR::R9  => 9,  GPR::R10 => 10, GPR::R11 => 11,
		GPR::R12 => 12, GPR::R13 => 13, GPR::R14 => 14, GPR::R15 => 15,
		GPR::R16 => 16, GPR::R17 => 17, GPR::R18 => 18, GPR::R19 => 19,
		GPR::R20 => 20, GPR::R21 => 21, GPR::R22 => 22, GPR::R23 => 23,
		GPR::R24 => 24, GPR::R25 => 25, GPR::R26 => 26, GPR::R27 => 27,
		GPR::R28 => 28, GPR::R29 => 29, GPR::R30 => 30, GPR::R31 => 31,
	};
}

pub type LabelOffsets = HashMap<CompactString, u64>;

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

fn resolve_offsets(lines: &mut [AsmLine]) -> Result<(LabelOffsets, usize), Error> {
	let mut label_offsets = HashMap::new();
	let mut offset = 0u64;

	for line in lines {
		let (len, align) = line_stats(&line);
		if !offset.is_multiple_of(align) {
			let new_offset = offset.next_multiple_of(align);
			line.pad = new_offset - offset;
			offset = new_offset;
		}
		if let Some(label) = &line.label {
			label_offsets.insert(label.name.clone(), offset);
		}
		offset += len;
	}

	Ok((label_offsets, offset as usize))
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

fn encode(lines: &mut [AsmLine], source: &Source) -> Result<Vec<u8>, Error> {
	let (label_offsets, len) = resolve_offsets(lines)?;
	let mut bin = Vec::with_capacity(len);

	for line in lines {
		if line.pad > 0 {
			bin.resize(bin.len() + line.pad as usize, 0);
		}
		match &line.kind {
			AsmLineKind::Const(cons) => encode_const(&mut bin, cons),
			AsmLineKind::Inst(inst) => {
				let inst = encode_inst(inst, &label_offsets, source)?;
				bin.extend_from_slice(&inst.to_le_bytes());
			}
		}
	}

	Ok(bin)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstBuilder<'a> {
	pub inst: u32,
	pub index: u8,
	pub source: &'a Source<'a>,
}

impl InstBuilder<'_> {
	pub fn new<'b>(source: &'b Source<'b>) -> InstBuilder<'b> {
		InstBuilder { inst: 0, index: 0, source }
	}
	pub fn finish(self) -> u32 {
		assert!(self.index == 32);
		self.inst
	}
	pub fn b(self, value: u32, len: u8) -> Self {
		Self {
			inst: bit_insert(self.inst, value, self.index, len),
			index: self.index + len,
			source: self.source,
		}
	}
	pub fn gpr(self, reg: GPR) -> Self {
		self.b(reg_code(reg) as u32, 5)
	}
	pub fn cond(self, operand: &Operand) -> Self {
		match operand.kind {
			OperandKind::GPR(reg) => self.b(reg_code(reg) as u32, 5),
			_ => self.b(0, 5),
		}
	}
	pub fn u_imd(self, value: u64, len: u8, span: Span) -> Result<Self, Error> {
		if value > (1 << len) - 1 {
			let src = self.source.source_of(span);
			return err!("number ({src}) out of range for u{len}_imd", (span, self.source));
		}
		Ok(self.b(value as u32, len))
	}
	pub fn s_imd(self, value: i64, len: u8, span: Span) -> Result<Self, Error> {
		if value > (1 << (len - 1)) - 1 || value < -(1 << (len - 1)) {
			let src = self.source.source_of(span);
			return err!("number ({src}) out of range for s{len}_imd", (span, self.source));
		}
		Ok(self.b(value as u32, len))
	}
}
