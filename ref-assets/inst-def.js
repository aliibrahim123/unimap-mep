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
let u6_imd = (name) => ({ name, type: "u6_imd", width: 6 });
let u12_imd = (name) => ({ name, type: "u12_imd", width: 12 });
let s12_imd = (name) => ({ name, type: "s12_imd", width: 12 });

let op_dpr = b4(1);
let op_dpi = b4(2);
let dpr_3reg = b4(0);
let dpr_2reg = b4(1);
let dpr_4reg = b4(2);
let dpr_2r_g0 = b8(0);
let dpr_3r_g0 = b4(0);
let dpr_3r_g1 = b4(1);

let r0 = { value: "00000", type: "gpr", width: 5 };
let sh_op = [{ name: "sh", width: 2 }, u6_imd("sh_amount")];
let sh_0 = [{ value: 0, type: "sh", width: 2 }, { value: 0, type: "sh_amount", width: 6 }];
let cw = { name: "cw", width: 2 };
let ca = { name: "ca", width: 1 };
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
	},

];