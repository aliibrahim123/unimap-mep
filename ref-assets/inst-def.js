let b1 = (value) => ({ value, width: 1 });
let b2 = (value) => ({ value, width: 2 });
let b3 = (value) => ({ value, width: 3 });
let b4 = (value) => ({ value, width: 4 });
let b5 = (value) => ({ value, width: 5 });
let b6 = (value) => ({ value, width: 6 });
let b8 = (value) => ({ value, width: 8 });

let gpr = (name) => ({ name, type: "gpr", width: 5 });
let gpr_pc = (name) => ({ name, type: "gpr_pc", width: 5 });
let cond = (name) => ({ name, type: "cond", width: 5 });
let u2_imd = (name) => ({ name, type: "u2_i", width: 2 });
let u3_imd = (name) => ({ name, type: "u3_imd", width: 3 });
let u6_imd = (name) => ({ name, type: "u6_imd", width: 6 });
let u12_imd = (name) => ({ name, type: "u12_imd", width: 12 });
let u16_imd = (name) => ({ name, type: "u16_imd", width: 16 });
let s9_imd = (name) => ({ name, type: "s9_imd", width: 8 });
let s10_imd = (name) => ({ name, type: "s10_imd", width: 10 });
let s12_imd = (name) => ({ name, type: "s12_imd", width: 12 });
let s19_imd = (name) => ({ name, type: "s19_imd", width: 19 });
let s24_imd = (name) => ({ name, type: "s24_imd", width: 24 });

let op_dpr = b4(1);
let op_dpi = b4(2);
let op_mem = b4(3);
let op_branch = b4(4);
let dpr_3reg = b4(0);
let dpr_2reg = b4(1);
let dpr_4reg = b4(2);
let dpr_2r_g0 = b8(0);
let dpr_3r_g0 = b4(0);
let dpr_3r_g1 = b4(1);
let dpr_3r_g2 = b4(2);

let r0 = { value: "00000", type: "gpr", width: 5 };
let sh = (value) => ({ value, type: "sh", width: 2 });
let sh_op = [{ name: "sh", width: 2 }, u6_imd("sh_amount")];
let cw = { name: "cw", width: 2 };
let ca = { name: "ca", width: 1 };
let sz = { name: "sz", width: 2 };
let logic_imd = [u6_imd("ones"), u6_imd("rot")];
let l0 = { name: "l0", width: 1 };

let x = (bits) => ({ value: "x".repeat(bits), width: bits });
let f1 = (name) => ({ name, width: 1 });
let f2 = (name) => ({ name, width: 2 });
let f3 = (name) => ({ name, width: 3 });
let f4 = (name) => ({ name, width: 4 });
let f5 = (name) => ({ name, width: 5 });
let f6 = (name) => ({ name, width: 6 });
let f12 = (name) => ({ name, width: 12 });

export default [
	{
		name: "add",
		fields: [op_dpr, b4(8), ...sh_op, b1(0), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "add_imd",
		fields: [op_dpi, b4(4), b2(0), gpr("dst"), gpr_pc("src1"), u12_imd("src2")]
	}, {
		name: "add_carry",
		fields: [op_dpr, dpr_4reg, b4(0), gpr("dst"), gpr("src1"), gpr("src2"), cond("carry")]
	}, {
		name: "add3",
		fields: [op_dpr, dpr_4reg, b4(2), gpr("dst"), gpr("src1"), gpr("src2"), gpr("src3")]
	}, {
		name: "sub",
		fields: [op_dpr, b4(9), ...sh_op, b1(0), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "sub_rev",
		fields: [op_dpr, b4(10), ...sh_op, b1(0), gpr("dst"), gpr("src2"), gpr("src1")]
	}, {
		name: "sub_imd",
		fields: [op_dpi, b4(4), b2(1), gpr("dst"), gpr("src1"), u12_imd("src2")]
	}, {
		name: "sub_rev_imd",
		fields: [op_dpi, b4(4), b2(2), gpr("dst"), gpr("src2"), u12_imd("src1")]
	}, {
		name: "sub_borrow",
		fields: [op_dpr, dpr_4reg, b4(1), gpr("dst"), gpr("src1"), gpr("src2"), cond("borrow")]
	}, {
		name: "mult",
		fields: [op_dpr, dpr_4reg, b4(5), gpr("dst"), r0, gpr("src1"), gpr("src2")]
	}, {
		name: "mult_full",
		fields: [op_dpr, dpr_4reg, b4(5), gpr("plow"), gpr("phigh"), gpr("src1"), gpr("src2")]
	}, {
		name: "madd",
		fields: [op_dpr, dpr_4reg, b4(3), gpr("dst"), gpr("src1"), gpr("src2"), gpr("src3")]
	}, {
		name: "msub",
		fields: [op_dpr, dpr_4reg, b4(4), gpr("dst"), gpr("src1"), gpr("src2"), gpr("src3")]
	}, {
		name: "div",
		fields: [op_dpr, dpr_4reg, b4(6), gpr("dst"), r0, gpr("src1"), gpr("src2")]
	}, {
		name: "rem",
		fields: [op_dpr, dpr_4reg, b4(6), r0, gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "div_full",
		fields: [op_dpr, dpr_4reg, b4(6), gpr("quo"), gpr("rem"), gpr("src1"), gpr("src2")]
	}, {
		name: "udiv",
		fields: [op_dpr, dpr_4reg, b4(7), gpr("dst"), r0, gpr("src1"), gpr("src2")]
	}, {
		name: "urem",
		fields: [op_dpr, dpr_4reg, b4(7), r0, gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "udiv_full",
		fields: [op_dpr, dpr_4reg, b4(7), gpr("quo"), gpr("rem"), gpr("src1"), gpr("src2")]
	}, {
		name: "cinc",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(7), gpr("dst"), gpr("cond"), gpr("src")]
	}, {
		name: "abs",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(2), gpr("dst"), gpr("src")]
	}, {
		name: "neg",
		fields: [op_dpr, b4(9), ...sh_op, b1(0), gpr("dst"), r0, gpr("src")]
	}, {
		name: "cneg",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(8), gpr("dst"), gpr("cond"), gpr("src")]
	}, {
		name: "se",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, sz, b4(3), gpr("dst"), gpr("src")]
	}, {
		name: "min",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(2), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "max",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(3), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "umin",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(4), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "umax",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(5), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "comp_eq",
		fields: [op_dpr, dpr_3reg, dpr_3r_g1, b3(0), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "comp_ne",
		fields: [op_dpr, dpr_3reg, dpr_3r_g1, b3(1), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "comp_gt",
		fields: [op_dpr, dpr_3reg, dpr_3r_g1, b3(2), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "comp_le",
		fields: [op_dpr, dpr_3reg, dpr_3r_g1, b3(3), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "ucomp_gt",
		fields: [op_dpr, dpr_3reg, dpr_3r_g1, b3(4), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "ucomp_le",
		fields: [op_dpr, dpr_3reg, dpr_3r_g1, b3(5), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "comp_eq_imd",
		fields: [op_dpi, b4(5), b1(0), ca, cond("dst"), gpr("src1"), s12_imd("src2")]
	}, {
		name: "comp_ne_imd",
		fields: [op_dpi, b4(5), b1(1), ca, cond("dst"), gpr("src1"), s12_imd("src2")]
	}, {
		name: "comp_gt_imd",
		fields: [op_dpi, b4(6), b1(0), ca, cond("dst"), gpr("src1"), s12_imd("src2")]
	}, {
		name: "ucomp_gt_imd",
		fields: [op_dpi, b4(6), b1(1), ca, cond("dst"), gpr("src1"), u12_imd("src2")]
	}, {
		name: "not",
		fields: [op_dpr, b4(14), ...sh_op, b1(1), gpr("dst"), r0, gpr("src")]
	}, {
		name: "cnot",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(6), gpr("dst"), gpr("cond"), gpr("src")]
	}, {
		name: "and",
		fields: [op_dpr, b4(8), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "or",
		fields: [op_dpr, b4(9), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "xor",
		fields: [op_dpr, b4(10), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "imply",
		fields: [op_dpr, b4(11), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "nand",
		fields: [op_dpr, b4(12), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "nor",
		fields: [op_dpr, b4(13), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "xnor",
		fields: [op_dpr, b4(14), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "bcr",
		fields: [op_dpr, b4(15), ...sh_op, b1(1), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "and_imd",
		fields: [op_dpi, b4(0), b1(0), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "or_imd",
		fields: [op_dpi, b4(0), b1(1), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "xor_imd",
		fields: [op_dpi, b4(1), b1(0), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "imply_imd",
		fields: [op_dpi, b4(1), b1(1), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "nand_imd",
		fields: [op_dpi, b4(2), b1(0), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "nor_imd",
		fields: [op_dpi, b4(2), b1(1), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "xnor_imd",
		fields: [op_dpi, b4(3), b1(0), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "bcr_imd",
		fields: [op_dpi, b4(3), b1(1), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "test_none",
		fields: [op_dpr, dpr_3reg, dpr_3r_g2, b3(0), cw, cond("dst"), gpr("src"), gpr("mask")]
	}, {
		name: "test_any",
		fields: [op_dpr, dpr_3reg, dpr_3r_g2, b3(1), cw, cond("dst"), gpr("src"), gpr("mask")]
	}, {
		name: "test_all",
		fields: [op_dpr, dpr_3reg, dpr_3r_g2, b3(2), cw, cond("dst"), gpr("src"), gpr("mask")]
	}, {
		name: "test_none_imd",
		fields: [op_dpi, b4(7), ca, l0, cond("dst"), gpr("src"), ...logic_imd]
	}, {
		name: "test_any_imd",
		fields: [op_dpi, b4(8), ca, l0, cond("dst"), gpr("src"), ...logic_imd]
	}, {
		name: "test_all_imd",
		fields: [op_dpi, b4(9), ca, l0, cond("dst"), gpr("src"), ...logic_imd]
	}, {
		name: "shl",
		fields: [op_dpr, dpr_4reg, b4(8), gpr("dst"), gpr("src"), r0, gpr("amount")]
	}, {
		name: "shl_imd",
		fields: [op_dpr, b4(9), sh(0), u6_imd("amount"), b1(1), gpr("dst"), r0, gpr("src")]
	}, {
		name: "shr",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(0), gpr("dst"), gpr("src"), gpr("amount")]
	}, {
		name: "shr_imd",
		fields: [op_dpr, b4(9), sh(1), u6_imd("amount"), b1(1), gpr("dst"), r0, gpr("src")]
	}, {
		name: "sar",
		fields: [op_dpr, dpr_3reg, dpr_3r_g0, b5(1), gpr("dst"), gpr("src"), gpr("amount")]
	}, {
		name: "sar_imd",
		fields: [op_dpr, b4(9), sh(2), u6_imd("amount"), b1(1), gpr("dst"), r0, gpr("src")]
	}, {
		name: "rol",
		fields: [op_dpr, dpr_4reg, b4(8), gpr("dst"), gpr("src"), gpr("src"), gpr("amount")]
	}, {
		name: "rol_imd",
		fields: [op_dpr, b4(9), sh(3), u6_imd("amount"), b1(1), gpr("dst"), r0, gpr("src")]
	}, {
		name: "fush",
		fields: [op_dpr, dpr_4reg, b4(8), gpr("dst"), gpr("src"), gpr("carry"), gpr("amount")]
	}, {
		name: "bfext",
		fields: [op_dpr, dpr_4reg, b4(9), gpr("dst"), gpr("src"), gpr("offset"), gpr("width")]
	}, {
		name: "bfext_imd",
		fields: [op_dpi, b4(12), b2(0), gpr("dst"), gpr("src"), u6_imd("offset"), u6_imd("width")]
	}, {
		name: "bfins",
		fields: [op_dpr, dpr_4reg, b4(10), gpr("dst"), gpr("src"), gpr("offset"), gpr("width")]
	}, {
		name: "bfins_imd",
		fields: [op_dpi, b4(12), b2(1), gpr("dst"), gpr("src"), u6_imd("offset"), u6_imd("width")]
	}, {
		name: "cnt",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(0), gpr("dst"), gpr("src")]
	}, {
		name: "cntz",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(1), gpr("dst"), gpr("src")]
	}, {
		name: "clz",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(4), gpr("dst"), gpr("src")]
	}, {
		name: "clo",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(5), gpr("dst"), gpr("src")]
	}, {
		name: "ctz",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(6), gpr("dst"), gpr("src")]
	}, {
		name: "cto",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(7), gpr("dst"), gpr("src")]
	}, {
		name: "cls",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(3), gpr("dst"), gpr("src")]
	}, {
		name: "rev",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, b6(8), gpr("dst"), gpr("src")]
	}, {
		name: "rev_parts",
		fields: [op_dpr, dpr_2reg, dpr_2r_g0, sz, b4(2), gpr("dst"), gpr("src")]
	}, {
		name: "mov",
		fields: [
			op_dpr, b4(9), sh(0), { value: 0, type: "sh_amount", width: 6 },
			b1(1), gpr("dst"), gpr("src"), r0
		]
	}, {
		name: "mov_imd",
		fields: [op_dpi, b4(10), b1(0), u2_imd("sh"), gpr("dst"), u16_imd("src")]
	}, {
		name: "mov_logic_imd",
		fields: [op_dpi, b4(0), b1(1), l0, gpr("dst"), r0, ...logic_imd]
	}, {
		name: "mov_neg_imd",
		fields: [op_dpi, b4(4), b2(1), gpr("dst"), r0, u12_imd("src")]
	}, {
		name: "mov_keep",
		fields: [op_dpi, b4(10), b1(1), u2_imd("sh"), gpr("dst"), u16_imd("src")]
	}, {
		name: "sel",
		fields: [op_dpr, dpr_4reg, b4(11), gpr("dst"), cond("cond"), gpr("src1"), gpr("src2")]
	}, {
		name: "sel_imd",
		fields: [op_dpi, b4(11), b1("s"), gpr("dst"), cond("cond"), gpr("src1"), s9_imd("src2")]
	}, {
		name: "ld_offset",
		fields: [op_mem, b2(3), sz, s19_imd("offset"), gpr("dst")]
	}, {
		name: "mem_base_offset",
		fields: [op_mem, b1("w"), b1(0), { name: "op" }, sz, gpr("reg"), gpr("base"), s12_imd("offset")]
	}, {
		name: "ld_base_offset",
		fields: [op_mem, b1("w"), b1(0), b2(0), sz, gpr("dst"), gpr("base"), s12_imd("offset")]
	}, {
		name: "st_base_offset",
		fields: [op_mem, b1("w"), b1(0), b2(1), sz, gpr("src"), gpr("base"), s12_imd("offset")]
	}, {
		name: "ld_s_base_offset",
		fields: [op_mem, b1("w"), b1(0), b2(2), sz, gpr("dst"), gpr("base"), s12_imd("offset")]
	}, {
		name: "mem_base_index",
		fields: [op_mem, b2(2), b6(0), { name: "op" }, sz, "s", gpr("reg"), gpr("base"), gpr("index")]
	}, {
		name: "ld_base_index",
		fields: [op_mem, b2(2), b6(0), b2(0), sz, "s", gpr("dst"), gpr("base"), gpr("index")]
	}, {
		name: "st_base_index",
		fields: [op_mem, b2(2), b6(0), b2(1), sz, "s", gpr("src"), gpr("base"), gpr("index")]
	}, {
		name: "ld_s_base_index",
		fields: [op_mem, b2(2), b6(0), b2(2), sz, "s", gpr("dst"), gpr("base"), gpr("index")]
	}, {
		name: "br",
		fields: [op_branch, b4(0), s24_imd("offset")]
	}, {
		name: "br_link",
		fields: [op_branch, b4(1), s19_imd("offset"), gpr("link")]
	}, {
		name: "br_cond",
		fields: [op_branch, "c", b3(1), s19_imd("offset"), cond("cond")]
	}, {
		name: "jmp_index",
		fields: [op_branch, b4(4), b4(0), b2(0), u3_imd("sh"), gpr("base"), gpr("index"), gpr("link")]
	}, {
		name: "jmp_offset",
		fields: [op_branch, b4(4), b4(1), s10_imd("offset"), gpr("base"), gpr("link")]
	}, {
		name: "halt",
		fields: [op_branch, b4(0), { value: 0, width: 24 }]
	},

	{
		name: "root_enc",
		fields: [{ name: "pgrp" }, x(28)]
	}, {
		name: "dpr_enc",
		fields: [op_dpr, f4("grp"), x(24)]
	}, {
		name: "3_regs_f1_enc",
		fields: [op_dpr, dpr_3reg, f4("sub_grp"), f5("op"), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "3_regs_f2_enc",
		fields: [op_dpr, dpr_3reg, f4("sub_grp"), f5("op"), gpr("dst"), cond("src1"), gpr("src2")]
	}, {
		name: "3_regs_f3_enc",
		fields: [op_dpr, dpr_3reg, f4("sub_grp"), f3("op"), cw, cond("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "2_regs_enc",
		fields: [op_dpr, dpr_2reg, b8(0), f6("op"), gpr("dst"), gpr("src")]
	}, {
		name: "4_regs_f1_enc",
		fields: [op_dpr, dpr_4reg, f4("op"), gpr("dst"), gpr("src1"), gpr("src2"), gpr("src3")]
	}, {
		name: "4_regs_f2_enc",
		fields: [op_dpr, dpr_4reg, f4("op"), gpr("dst"), gpr("src1"), gpr("src2"), cond("src3")]
	}, {
		name: "4_regs_f3_enc",
		fields: [op_dpr, dpr_4reg, f4("op"), gpr("dst"), cond("src1"), gpr("src2"), gpr("src3")]
	}, {
		name: "shift_enc",
		fields: [op_dpr, f3("op2"), b1(1), ...sh_op, f1("o1"), gpr("dst"), gpr("src1"), gpr("src2")]
	}, {
		name: "dpi_enc",
		fields: [op_dpi, f4("grp"), x(24)]
	}, {
		name: "dpi_logic_enc",
		fields: [op_dpi, f2("op1"), b2(0), f1("o2"), l0, gpr("dst"), gpr("src1"), ...logic_imd]
	}, {
		name: "dpi_arith_enc",
		fields: [op_dpi, b4(4), f2("op"), gpr("dst"), gpr_pc("src1"), u12_imd("src2")]
	}, {
		name: "dpi_comp_enc",
		fields: [op_dpi, f4("grp"), f1("op"), ca, cond("dst"), gpr("src1"), f12("src2_imd")]
	}, {
		name: "dpi_bit_test_enc",
		fields: [op_dpi, f4("grp"), ca, l0, cond("dst"), gpr("src"), ...logic_imd]
	}, {
		name: "dpi_move_wide_enc",
		fields: [op_dpi, b4(10), f1("op"), u2_imd("sh"), gpr("dst"), u16_imd("src")]
	}, {
		name: "dpi_bitfield_enc",
		fields: [op_dpi, b4(12), f2("op"), gpr("dst"), gpr("src"), u6_imd("offset"), u6_imd("width")]
	}, {
		name: "mem_enc",
		fields: [op_mem, f2("amod"), x(26)]
	}, {
		name: "mem_base_offset_enc",
		fields: [op_mem, b1("w"), b1(0), f2("op"), sz, gpr("reg"), gpr("base"), s12_imd("offset")]
	}, {
		name: "mem_base_index_enc",
		fields: [op_mem, b2(2), b6(0), f2("op"), sz, f1("s"), gpr("reg"), gpr("base"), gpr("index")]
	}, {
		name: "br_enc",
		fields: [op_branch, f4("op1"), f4("op2"), x(20)]
	}
];