use crate::{
	encoder::{InstBuilder, LabelOffsets},
	parser::{Inst, Mnemonic, OperandKind},
	tokenizer::Source,
	utils::{Error, err},
};

fn undefined_instruction<T>(mnemonic: &Mnemonic, source: &Source) -> Result<T, Error> {
	err!("undefined instruction \"{mnemonic}\"", (mnemonic.span(), source))
}
fn invalid_operands<T>(inst: &Inst, source: &Source) -> Result<T, Error> {
	let mnemonic = &inst.mnemonic;
	err!("invalid operands for \"{mnemonic}\"", (inst.span, source))
}

const OP_DPR: u32 = 1;

pub fn encode_inst(
	inst: &Inst, label_offsets: &LabelOffsets, source: &Source,
) -> Result<u32, Error> {
	use OperandKind::*;
	let Inst { mnemonic, operands, .. } = inst;
	let oprands = operands.iter().map(|o| (&o.kind, o.span)).collect::<Vec<_>>();
	let oprands = &oprands[..];
	let build = InstBuilder::new(source);

	if mnemonic.segments.len() == 1 {
		Ok(match mnemonic.segments[0].name.as_str() {
			"add" => match oprands {
				[(GPR(dst), _), (GPR(src1), _), (GPR(src2), _)] => {
					build.b(OP_DPR, 4).b(8, 4).b(0, 9).gpr(*dst).gpr(*src1).gpr(*src2).finish()
				}
				_ => return invalid_operands(inst, source),
			},
			_ => return undefined_instruction(mnemonic, source),
		})
	} else {
		undefined_instruction(mnemonic, source)
	}
}
