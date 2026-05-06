use crate::{
	encoder::{InstBuilder, LabelOffsets},
	parser::{Ident, Inst, OperandKind, Reg, Shift},
	tokenizer::Source,
	utils::{Error, err},
};

fn undefined_instruction<T>(mnemonic: &Ident, source: &Source) -> Result<T, Error> {
	err!("undefined instruction \"{mnemonic}\"", (mnemonic.span, source))
}
fn invalid_operands<T>(inst: &Inst, source: &Source) -> Result<T, Error> {
	let mnemonic = &inst.mnemonic;
	err!("invalid operands for \"{mnemonic}\"", (inst.span, source))
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
const DPR_3R_G3: u8 = 2;

const SH_0: Shift = Shift::SHL(0);

pub fn encode_inst(
	inst: &Inst, label_offsets: &LabelOffsets, source: &Source,
) -> Result<u32, Error> {
	use OperandKind::*;
	let Inst { mnemonic, operands, .. } = inst;
	let operands = operands.iter().map(|o| (&o.kind, o.span)).collect::<Vec<_>>();
	let operands = &operands[..];
	let mut build = InstBuilder::new(source);

	Ok(match mnemonic.name.as_str() {
		"add" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build.b4(OP_DPR).b4(8).shift(SH_0).b1(0).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (ShReg(src2, shift), _)] => {
				build.b4(OP_DPR).b4(8).shift(*shift).b1(0).gpr(*dst).gpr(*src1).gpr(*src2).finish()
			}
			[(GPR(dst), _), (GPR(src1) | PC(src1), span1), (UImd(src2), span2)] => {
				build = build.b4(OP_DPI).b4(4).b2(0).gpr(*dst);
				build.gpr_pc(*src1, *span1)?.u_imd(*src2, 12, *span2)?.finish()
			}
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(src3), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(2).gpr(*dst).gpr(*src1);
				build.gpr(*src2).gpr(*src3).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"add.carry" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(carry) | C0(carry), span1)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(0).gpr(*dst).gpr(*src1);
				build.gpr(*src2).cond(*carry, *span1)?.finish()
			}
			_ => return invalid_operands(inst, source),
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
			[(GPR(dst), _), (GPR(src1), _), (UImd(src2), span2)] => {
				build.b4(OP_DPI).b4(4).b2(1).gpr(*dst).gpr(*src1).u_imd(*src2, 12, *span2)?.finish()
			}
			[(GPR(dst), _), (UImd(src1), span2), (GPR(src2), _)] => {
				build.b4(OP_DPI).b4(4).b2(2).gpr(*dst).gpr(*src2).u_imd(*src1, 12, *span2)?.finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"sub.borrow" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(borrow) | C0(borrow), span1)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(1).gpr(*dst).gpr(*src1);
				build.gpr(*src2).cond(*borrow, *span1)?.finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"mult" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(5).gpr(*dst).gpr(Reg::R0);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"mult.full" => match operands {
			[(GPR(plow), _), (GPR(phigh), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(5).gpr(*plow).gpr(*phigh);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"madd" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(src3), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(3).gpr(*dst).gpr(*src1);
				build.gpr(*src2).gpr(*src3).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"msub" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _), (GPR(src3), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(4).gpr(*dst).gpr(*src1);
				build.gpr(*src2).gpr(*src3).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"div" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(6).gpr(*dst).gpr(Reg::R0);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"rem" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(6).gpr(Reg::R0).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"div.full" => match operands {
			[(GPR(quo), _), (GPR(rem), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(6).gpr(*quo).gpr(*rem);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"udiv" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(7).gpr(*dst).gpr(Reg::R0);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"urem" => match operands {
			[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(7).gpr(Reg::R0).gpr(*dst);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"udiv.full" => match operands {
			[(GPR(quo), _), (GPR(rem), _), (GPR(src1), _), (GPR(src2), _)] => {
				build = build.b4(OP_DPR).b4(DPR_4REG).b4(7).gpr(*quo).gpr(*rem);
				build.gpr(*src1).gpr(*src2).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		"cinc" => match operands {
			[(GPR(dst), _), (C0(cond) | GPR(cond), span1), (GPR(src), _)] => {
				build = build.b4(OP_DPR).b4(DPR_3REG).b4(DPR_3R_G0).b5(7).gpr(*dst);
				build.cond(*cond, *span1)?.gpr(*src).finish()
			}
			_ => return invalid_operands(inst, source),
		},
		_ => return undefined_instruction(mnemonic, source),
	})

	/*  */
}
