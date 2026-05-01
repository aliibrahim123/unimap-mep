let b1 = (value) => ({ value, width: 1 });
let b2 = (value) => ({ value, width: 2 });
let b4 = (value) => ({ value, width: 4 });
let gpr = (name) => ({ name, type: "gpr", width: 5 });
let gpr_pc = (name) => ({ name, type: "gpr_pc", width: 5 });
let cond = (name) => ({ name, type: "cond", width: 5 });
let u6_imd = (name) => ({ name, type: "u6_imd", width: 6 });
let u12_imd = (name) => ({ name, type: "u12_imd", width: 12 });
let r0 = { value: "00000", type: "gpr", width: 5 };
let op_dpr = b4(1);
let op_dpi = b4(2);
let dpr_4reg = b4(2);
let sh_op = [{ name: "sh", width: 2 }, u6_imd("sh_amount")];
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
	}

];