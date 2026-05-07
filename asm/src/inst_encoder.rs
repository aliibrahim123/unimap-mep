use crate::{
	encoder::{InstBuilder, LabelIndexes},
	parser::{Ident, Inst, Operand, OperandKind, Reg::R0, Shift},
	tokenizer::{Source, Span},
	utils::{Error, err},
};

fn undefined_instruction<T>(mnemonic: &Ident, source: &Source) -> Result<T, Error> {
	err!("undefined instruction \"{mnemonic}\"", (mnemonic.span, source))
}
fn invalid_operands<T>(mnemonic: &Ident, span: Span, source: &Source) -> Result<T, Error> {
	err!("invalid operands for \"{mnemonic}\"", (span, source))
}

const OP_DPR: u8 = 1;
const OP_DPI: u8 = 2;
const OP_MEM: u8 = 3;
const OP_BRANCH: u8 = 4;
const DPR_3REG: u8 = 0;
const DPR_2REG: u8 = 1;
const DPR_4REG: u8 = 2;
const DPR_2R_G0: u8 = 0;
const DPR_3R_G0: u8 = 0;
const DPR_3R_G1: u8 = 1;
const DPR_3R_G2: u8 = 2;

const SH_0: Shift = Shift::SHL(0);

fn resolve_label(
	label: &Ident, inst_index: u64, label_indexes: &LabelIndexes, source: &Source,
) -> Result<i64, Error> {
	let Some(index) = label_indexes.get(&label.name) else {
		return err!("undefined label \"{label}\"", (label.span, source));
	};
	Ok(*index as i64 - inst_index as i64)
}
fn prepare_operands(
	operands: Vec<Operand>, inst_index: u64, label_indexes: &LabelIndexes, source: &Source,
) -> Result<Vec<(OperandKind, Span)>, Error> {
	use OperandKind::*;
	let mut res = Vec::with_capacity(operands.len());
	for op in operands {
		let kind = match op.kind {
			Label(label) => Imd(resolve_label(&label, inst_index, label_indexes, source)?),
			OffsetLabel(label) => {
				Offset(resolve_label(&label, inst_index, label_indexes, source)?, label.span)
			}
			other => other,
		};
		res.push((kind, op.span));
	}
	Ok(res)
}

pub fn encode_inst(
	inst: Inst, inst_index: u64, label_indexes: &LabelIndexes, source: &Source,
) -> Result<u32, Error> {
	use OperandKind::*;
	let Inst { mnemonic, operands, .. } = inst;
	let operands = &prepare_operands(operands, inst_index, label_indexes, source)?[..];
	let mut build = InstBuilder::new(source);

	Ok(match mnemonic.name.as_str() {
		"add" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(8).shift(SH_0).b1(0).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(8).shift(*shift).b1(0).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1) | PC(src1), span1), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(4).b2(0).gpr(*dst);
				build.gpr_pc(*src1, *span1)?.u_imd(*src2, 12, *span2)?.finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(src3), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(2).gpr(*dst).gpr(*src1);
				build.gpr(*src2).gpr(*src3).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"add.carry" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(carry) | C0(carry), span1)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(0).gpr(*dst).gpr(*src1);
				build.gpr(*src2).cond(*carry, *span1)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"sub" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(9).shift(SH_0).b1(0).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(9).shift(*shift).b1(0).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (ShReg(src1, shift), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(10).shift(*shift).b1(0).gpr(*dst).gpr(*src2).gpr(*src1).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(4).b2(1).gpr(*dst).gpr(*src1);
				build.u_imd(*src2, 12, *span2)?.finish()
			}
			[(GPR(dst), _), (Imd(src1), span2), (GPR(src2), _)] => {
				build = build.b4(OP_DPI).b4(4).b2(2).gpr(*dst).gpr(*src2);
				build.u_imd(*src1, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"sub.borrow" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(borrow) | C0(borrow), span1)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(1).gpr(*dst).gpr(*src1);
				build.gpr(*src2).cond(*borrow, *span1)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"mult" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(5).gpr(*dst).gpr(R0);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"mult.full" => match operands {
			[(GPR(plow), _), (GPR(phigh), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(5).gpr(*plow).gpr(*phigh);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"madd" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(src3), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(3).gpr(*dst).gpr(*src1);
				build.gpr(*src2).gpr(*src3).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"msub" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(src3), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(4).gpr(*dst).gpr(*src1);
				build.gpr(*src2).gpr(*src3).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"div" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(6).gpr(*dst).gpr(R0);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"rem" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(6).gpr(R0).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"div.full" => match operands {
			[(GPR(quo), _), (GPR(rem), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(6).gpr(*quo).gpr(*rem);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"udiv" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(7).gpr(*dst).gpr(R0);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"urem" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(7).gpr(R0).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"udiv.full" => match operands {
			[(GPR(quo), _), (GPR(rem), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(7).gpr(*quo).gpr(*rem);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cinc" => match operands {
			[(GPR(dst), _), (C0(cond) | GPR(cond), span1), (GPR(src), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(7).gpr(*dst);
				build.cond(*cond, *span1)?.gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"abs" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(2).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"neg" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(9).shift(SH_0).b1(0).gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			[(GPR(dst), _), (ShReg(src, shift), _)] => {
				build.b4(OP_DPR).b4(9).shift(*shift).b1(0).gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cneg" => match operands {
			[(GPR(dst), _), (C0(cond) | GPR(cond), span1), (GPR(src), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(8).gpr(*dst);
				build.cond(*cond, *span1)?.gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"se.8" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(15).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"se.16" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(14).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"se.32" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(13).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"min" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				let build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(2).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"max" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				let build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(3).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"umin" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				let build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(4).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"umax" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				let build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(5).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.eq" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(0).b2(0);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(5).b1(0).b1(0).cond(*dst, *span1)?;
				build.gpr(*src1).s_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.eq.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(0).b2(1);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(5).b1(0).b1(1).cond(*dst, *span1)?;
				build.gpr(*src1).s_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.eq.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(0).b2(3);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.ne" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(1).b2(0);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(5).b1(1).b1(0).cond(*dst, *span1)?;
				build.gpr(*src1).s_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.ne.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(1).b2(1);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(5).b1(1).b1(1).cond(*dst, *span1)?;
				build.gpr(*src1).s_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.ne.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(1).b2(3);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.gt" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(2).b2(0);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(6).b1(0).b1(0).cond(*dst, *span1)?;
				build.gpr(*src1).s_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.gt.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(2).b2(1);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(6).b1(0).b1(1).cond(*dst, *span1)?;
				build.gpr(*src1).s_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.gt.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(2).b2(3);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.le" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(3).b2(0);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.le.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(3).b2(1);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"comp.le.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(3).b2(3);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ucomp.gt" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(4).b2(0);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(6).b1(1).b1(0).cond(*dst, *span1)?;
				build.gpr(*src1).u_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ucomp.gt.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(4).b2(1);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(6).b1(1).b1(1).cond(*dst, *span1)?;
				build.gpr(*src1).u_imd(*src2, 12, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ucomp.gt.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(4).b2(3);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ucomp.le" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(5).b2(0);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ucomp.le.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(5).b2(1);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ucomp.le.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G1).b3(5).b2(3);
				build.cond(*dst, *span1)?.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"not" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(14).shift(SH_0).b1(1).gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			[(GPR(dst), _), (ShReg(src, shift), _)] => {
				build.b4(OP_DPR).b4(14).shift(*shift).b1(1).gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cnot" => match operands {
			[(GPR(dst), _), (C0(cond) | GPR(cond), span1), (GPR(src), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(6).gpr(*dst);
				build.cond(*cond, *span1)?.gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"and" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(8).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(8).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(0).b1(0).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"or" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(9).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(9).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(0).b1(1).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"xor" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(10).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(10).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(1).b1(0).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"imply" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(11).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(11).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(1).b1(1).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"nand" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(12).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(12).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(2).b1(0).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"nor" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(13).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(13).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(2).b1(1).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"xnor" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(14).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(14).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(3).b1(0).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"bcr" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(15).shift(SH_0).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(15).shift(*shift).b1(1).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(3).b1(1).limd_l0(*level).gpr(*dst);
				build.gpr(*src1).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.none" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(0).b2(0);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(7).b1(0).limd_l0(*level).cond(*dst, *span1)?;
				build.gpr(*src).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.none.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(0).b2(1);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(7).b1(1).limd_l0(*level).cond(*dst, *span1)?;
				build.gpr(*src).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.none.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(0).b2(3);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.any" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(1).b2(0);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(8).b1(0).limd_l0(*level).cond(*dst, *span1)?;
				build.gpr(*src).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.any.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(1).b2(1);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(8).b1(1).limd_l0(*level).cond(*dst, *span1)?;
				build.gpr(*src).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.any.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(1).b2(3);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.all" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(2).b2(0);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(9).b1(0).limd_l0(*level).cond(*dst, *span1)?;
				build.gpr(*src).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.all.and" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(2).b2(1);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(9).b1(1).limd_l0(*level).cond(*dst, *span1)?;
				build.gpr(*src).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"test.all.or" => match operands {
			[(C0(dst) | GPR(dst), span1), (GPR(src), _), (GPR(mask), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G2).b3(2).b2(3);
				build.cond(*dst, *span1)?.gpr(*src).gpr(*mask).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"shl" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(amount), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(8).gpr(*dst).gpr(*src);
				build.gpr(R0).gpr(*amount).finish()
			}
			[(GPR(dst), _), (GPR(src), _), (Imd(amount), span1)] => {
				build = build.b4(OP_DPR).b4(9).b2(0).u_imd(*amount, 6, *span1)?.b1(1);
				build.gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"shr" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(amount), _)] => {
				let build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(0).gpr(*dst);
				build.gpr(*src).gpr(*amount).finish()
			}
			[(GPR(dst), _), (GPR(src), _), (Imd(amount), span1)] => {
				build = build.b4(OP_DPR).b4(9).b2(1).u_imd(*amount, 6, *span1)?.b1(1);
				build.gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"sar" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(amount), _)] => {
				let build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(1).gpr(*dst);
				build.gpr(*src).gpr(*amount).finish()
			}
			[(GPR(dst), _), (GPR(src), _), (Imd(amount), span1)] => {
				build = build.b4(OP_DPR).b4(9).b2(2).u_imd(*amount, 6, *span1)?.b1(1);
				build.gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"rol" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(amount), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(8).gpr(*dst).gpr(*src);
				build.gpr(*src).gpr(*amount).finish()
			}
			[(GPR(dst), _), (GPR(src), _), (Imd(amount), span1)] => {
				build = build.b4(OP_DPR).b4(9).b2(3).u_imd(*amount, 6, *span1)?.b1(1);
				build.gpr(*dst).gpr(R0).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"fush" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(carry), _), (GPR(amount), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(8).gpr(*dst).gpr(*src);
				build.gpr(*carry).gpr(*amount).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"bfext" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(offset), _), (GPR(width), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(9).gpr(*dst).gpr(*src);
				build.gpr(*offset).gpr(*width).finish()
			}
			[(GPR(dst), _), (GPR(src), _), (Imd(offset), span1), (Imd(width), span2)] => {
				if *width == 0 {
					let src = source.source_of(*span2);
					return err!("width ({src}) cannot be 0", (*span2, source));
				}
				build = build.b4(OP_DPI).b4(12).b2(0).gpr(*dst).gpr(*src);
				build.u_imd(*offset, 6, *span1)?.u_imd(*width, 6, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"bfins" => match operands {
			[(GPR(dst), _), (GPR(src), _), (GPR(offset), _), (GPR(width), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(10).gpr(*dst).gpr(*src);
				build.gpr(*offset).gpr(*width).finish()
			}
			[(GPR(dst), _), (GPR(src), _), (Imd(offset), span1), (Imd(width), span2)] => {
				if *width == 0 {
					let src = source.source_of(*span2);
					return err!("width ({src}) cannot be 0", (*span2, source));
				}
				build = build.b4(OP_DPI).b4(12).b2(1).gpr(*dst).gpr(*src);
				build.u_imd(*offset, 6, *span1)?.u_imd(*width, 6, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cnt" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(0).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cntz" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(1).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"clz" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(4).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"clo" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(5).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ctz" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(6).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cto" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(7).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"cls" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(3).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"rev" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(8).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"rev.32" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(9).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"rev.16" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(10).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"rev.8" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(DPR_2REG).b8(DPR_2R_G0).b6(11).gpr(*dst).gpr(*src).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"mov" => match operands {
			[(GPR(dst), _), (GPR(src), _)] => {
				build.b4(OP_DPR).b4(9).shift(SH_0).b1(1).gpr(*dst).gpr(*src).gpr(R0).finish()
			}
			[(GPR(dst), _), (Imd(src), span1)] if *src < 0 => {
				let build = build.b4(OP_DPI).b4(4).b2(1).gpr(*dst).gpr(R0);
				build.u_imd(src.abs(), 12, *span1)?.finish()
			}
			[(GPR(dst), _), (Imd(src), span1)] => {
				build.b4(OP_DPI).b4(10).b3(0).gpr(*dst).u_imd(*src, 16, *span1)?.finish()
			}
			[(GPR(dst), _), (Imd(src), span1), (Imd(sh), span2)] => {
				build = build.b4(OP_DPI).b4(10).b1(0).u_imd(*sh, 2, *span2)?;
				build.gpr(*dst).u_imd(*src, 16, *span1)?.finish()
			}
			[(GPR(dst), _), (LogicImd { level, ones, rot }, _)] => {
				let build = build.b4(OP_DPI).b4(0).b1(1).limd_l0(*level).gpr(*dst);
				build.gpr(R0).logic_imd(*level, *ones, *rot).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"mov.keep" => match operands {
			[(GPR(dst), _), (Imd(src), span1), (Imd(sh), span2)] => {
				build = build.b4(OP_DPI).b4(10).b1(1).u_imd(*sh, 2, *span2)?;
				build.gpr(*dst).u_imd(*src, 16, *span1)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"sel" => match operands {
			[(GPR(dst), _), (C0(cond) | GPR(cond), span1), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(11).gpr(*dst).cond(*cond, *span1)?;
				build.gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (C0(cond) | GPR(cond), span1), (GPR(src1), _), (Imd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(11).b1((*src2 < 0) as u8).gpr(*dst);
				build = build.cond(*cond, *span1)?.gpr(*src1);
				build.s_imd_no_sbit(*src2, 8, *span2)?.finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld" => match operands {
			[(GPR(dst), _), (Offset(offset, span1), _)] => {
				build = build.b4(OP_MEM).b2(3).b2(0).scaled_s_imd(*offset, 19, 3, *span1)?;
				build.gpr(*dst).finish()
			}
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(0).b2(0).gpr(*dst);
				build.gpr(*base).scaled_s_imd(*offset, 12, 3, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(0).b2(0).ind_sh(*shift, 3, *shift_span)?;
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld.32" => match operands {
			[(GPR(dst), _), (Offset(offset, span1), _)] => {
				build = build.b4(OP_MEM).b2(3).b2(1).scaled_s_imd(*offset, 19, 2, *span1)?;
				build.gpr(*dst).finish()
			}
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(0).b2(1).gpr(*dst);
				build.gpr(*base).scaled_s_imd(*offset, 12, 3, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(0).b2(1).ind_sh(*shift, 2, *shift_span)?;
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld.16" => match operands {
			[(GPR(dst), _), (Offset(offset, span1), _)] => {
				build = build.b4(OP_MEM).b2(3).b2(2).scaled_s_imd(*offset, 19, 1, *span1)?;
				build.gpr(*dst).finish()
			}
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(0).b2(2).gpr(*dst);
				build.gpr(*base).scaled_s_imd(*offset, 12, 1, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(0).b2(2).ind_sh(*shift, 1, *shift_span)?;
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld.8" => match operands {
			[(GPR(dst), _), (Offset(offset, span1), _)] => {
				build.b4(OP_MEM).b2(3).b2(3).s_imd(*offset, 19, *span1)?.gpr(*dst).finish()
			}
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(0).b2(3).gpr(*dst);
				build.gpr(*base).s_imd(*offset, 12, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift: 0, .. }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(0).b2(3).b1(0);
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"st" => match operands {
			[(GPR(src), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(1).b2(0).gpr(*src);
				build.gpr(*base).scaled_s_imd(*offset, 12, 3, *offset_span)?.finish()
			}
			[(GPR(src), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(1).b2(0).ind_sh(*shift, 3, *shift_span)?;
				build.gpr(*src).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"st.32" => match operands {
			[(GPR(src), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(1).b2(1).gpr(*src);
				build.gpr(*base).scaled_s_imd(*offset, 12, 2, *offset_span)?.finish()
			}
			[(GPR(src), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(1).b2(1).ind_sh(*shift, 2, *shift_span)?;
				build.gpr(*src).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"st.16" => match operands {
			[(GPR(src), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(1).b2(2).gpr(*src);
				build.gpr(*base).scaled_s_imd(*offset, 12, 1, *offset_span)?.finish()
			}
			[(GPR(src), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(1).b2(2).ind_sh(*shift, 1, *shift_span)?;
				build.gpr(*src).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"st.8" => match operands {
			[(GPR(src), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(1).b2(3).gpr(*src);
				build.gpr(*base).s_imd(*offset, 12, *offset_span)?.finish()
			}
			[(GPR(src), _), (BaseIndex { base, index, shift: 0, .. }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(1).b2(3).b1(0);
				build.gpr(*src).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld.s32" => match operands {
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(2).b2(1).gpr(*dst);
				build.gpr(*base).scaled_s_imd(*offset, 12, 3, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(2).b2(1).ind_sh(*shift, 2, *shift_span)?;
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld.s16" => match operands {
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(2).b2(2).gpr(*dst);
				build.gpr(*base).scaled_s_imd(*offset, 12, 1, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift, shift_span }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(2).b2(2).ind_sh(*shift, 1, *shift_span)?;
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		"ld.s8" => match operands {
			[(GPR(dst), _), (BaseOffset { base, offset, offset_span, writeback }, _)] => {
				build = build.b4(OP_MEM).b1(0).b1(*writeback as u8).b2(2).b2(3).gpr(*dst);
				build.gpr(*base).s_imd(*offset, 12, *offset_span)?.finish()
			}
			[(GPR(dst), _), (BaseIndex { base, index, shift: 0, .. }, _)] => {
				build = build.b4(OP_MEM).b2(2).b6(0).b2(2).b2(3).b1(0);
				build.gpr(*dst).gpr(*base).gpr(*index).finish()
			}
			_ => return invalid_operands(&mnemonic, inst.span, source),
		},
		_ => return undefined_instruction(&mnemonic, source),
	})

	/* */
}
