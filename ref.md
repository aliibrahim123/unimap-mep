# MEP
MEP (Mapped Expressions Processor) is an advance 64-bit RISC CPU implemented in Unimap.

MEP is a little-endian Von Neumann CPU. It features the standard characteristics of a RISC CPU, but also includes a wide range of powerful operations and compound instructions typically found in modern ISAs.

This document serves as the architectural reference for MEP, describing its assembly language and detailing its instruction encoding.

# Execution Model
## Data Types
MEP processes binary data in little-endian order, accessing bytes and bits from least significant to most significant.

Its native word size is 64 bits, which is the size of the general-purpose registers, memory addresses, and the data width of most common operations.

MEP supports unsigned and signed two's complement integers at 8, 16, 32, and 64-bit sizes.

## Registers
The majority of instructions operate on general-purpose registers (`gpr`). MEP provides 31 64-bit general-purpose registers named `r1` through `r31`.

These registers can be used freely for any purpose and are accessible to all instructions.

`r0` (the zero register) is a general-purpose register that is hardcoded to 0. Any read from it will resolve to 0, and all writes to it are discarded.

`pc` (program counter) is an internal 64-bit register that holds the memory address of the currently executing instruction. It is not accessible or modifiable by normal instructions, it is only accessible through special instructions.

### Condition Registers
MEP doesn't have a traditional flags register, instead, all GPRs can be used as condition registers.

When evaluated as a condition, a zero value is `false`, while any non-zero value is `true`.

Using `r0` as a condition register redirects to a 1-bit `c0` register. This allows to holding a condition without overwriting another register.

## Memory
Memory in MEP is byte-addressable with a 64-bit address space.

All fundamental data types are stored in memory in little-endian order and must be aligned to their respective sizes.

MEP utilizes a load-store architecture. All memory accesses are performed solely by `ld` and `st` instructions.

# Assembly Language
The assembly language is a human-readable form of MEP executable code.
## Basic Syntax
This section uses the [gramex meta language](https://docs.rs/gramex/latest/gramex/docs/gram_ref/index.html), and expects the file to be valid UTF-8.

Whitespace is insignificant and is only used to separate tokens. The whitespace characters are: space ` `, horizontal tab `\t`, and carriage return `\r`.

The line feed `\n` is used as a separator between instructions.

#### Comments
```gramex
let comment = "//" !"\n"* "\n"? | "/*" !"*/"* "*/";
```
MEP assembly supports line and block comments using their respective C-style syntax.

They are ignored by the parser and do not provide any semantic meaning.

#### Identifiers
```gramex
let ident = ("a".."z" | "A".."Z") ("a".."z" | "A".."Z" | "0".."9" | "_")*;
```
Identifiers are used as names for instructions, registers, labels, and constants.

#### Numbers
```gramex
let hex_dg = "0".."9" | "a".."f" | "A".."F";
let nb = "0b" ("0" | "1") ("_"? ("0" | "1"))* | "0x" hex_dg ("_"? hex_dg)* | "0".."9" ("_"? "0".."9")*;
```
Numbers are unsigned integers used as offsets, immediates, and constants.

They can be written in decimal, binary, or hexadecimal, with an optional `_` as a separator for readability.

```
123 0b1001_0101 0xff_AA
```

## Top-Level Structure
```gramex
let file = list<label_decl? "\n"* (inst | const), "\n"+>
```
An assembly file consists of instructions and constants separated by newlines. Each instruction or constant can be prefixed with a label.

The assembler encodes each instruction and constant into its binary form, then lays them out sequentially starting at address 0.

```
start: 
	ld.8 r1, [one]
	mov r2, 2
	add r3, r1, r2

one: u8 0x01
```

### Labels
```gramex
let label_decl = ident ":"
```
Labels are used to reference an instruction or constant by name inside immediates and offsets.

They resolve to the offset of the referenced item relative to the current instruction.

```
data: u32 0x12345678
loop_start: 
	ld.32 r1, [data]
```

## Instructions
```gramex
let mnemonic = list<ident | nb, ".">;
let inst = mnemonic list<oprand, ",">?;
```
An instruction consists of its mnemonic followed by its operands.

Instruction mnemonics are composed of a `.`-separated list of identifiers.

Each instruction can have zero or more operands separated by `,`.

```
add r3, r1, r2
```

### Instruction Declaration
```gramex
let inst_decl = mnemonic ("{" list<"_" | "." (ident | nb), ","> "}")? list<ident "?"? ":" oprand_type, ",">?;
```
Instruction declarations found in this document are composed of the instruction mnemonic followed by its operand declarations separated by `,`.

An instruction declaration can define multiple sub-instructions or modified variants by suffixing the mnemonic with a variant declaration.

The variant declaration is a comma-separated list of variant suffixes enclosed in curly brackets. A suffix can be a `_` (default variant) or a `.` followed by an identifier.

An operand declaration consists of the operand identifier followed by its type. An operand is optional if its identifier is suffixed with `?`.

An instruction mnemonic can be overloaded depending on its operands.

```
add dst:gpr, src1:gpr, src2:u12_imd
comp.eq{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```

### Operands
```gramex
let oprand = reg | sh_reg | imd | label | address;
let oprand_type = reg_type | "sh_reg" | imd_type | address_type; 
```
An operand encodes a storage location, value, or option that the instruction takes or performs on.

Operands come in different forms: they can be registers, immediates, memory locations, or labels.

```
add r3, r1, r2 shl 3
comp.eq c0, r3, +10
br.true c0, case_1
ld r4, [r1 += 0x10]
```

### Register Operands
```gramex
let gpr = "r" ("0".."9" | ("1" | "2") "0".."9" | "30" | "31");
let pc = "pc";
let c0 = "c0";
let reg = gpr | pc | c0;
let reg_type = "gpr" | "cond" | "gpr_pc";
```
Register operands are the most common operand types. They come in different categories:
- `gpr`: Any general-purpose register.
- `cond`: A register holding a condition, `c0` and any general-purpose register except `r0`.
- `gpr_pc`: `pc` and any general-purpose register except `r0`.

```
add r3, r1, r2
comp.eq c0, r1, r2
add r3, pc, +4
```

### Shifted Register Operands
```gramex
let sh_reg = gpr | gpr ("shl" | "shr" | "sar" | "rol") nb; 
```
Shifted register operands are general-purpose registers shifted by a constant encoded as a 6-bit immediate.

The supported shifts are logical left (`shl`), logical right (`shr`), arithmetic right (`sar`), and rotate left (`rol`).

The shift part can be omitted, resulting in `shl 0`.

```
add r3, r1, r2 shl 4
or r4, r1, r3 rol 31
```

### Immediate Operands
```gramex
let imd = ("+" | "-")? nb | logic_imd;
let imd_type = 
	"u2_imd" | "u3_imd" | "u6_imd" | "u12_imd" | "u16_imd"
	| "s9_imd" | "s10_imd" | "s12_imd" | "s19_imd" | "s24_imd" 
	| "logic_imd"
;
```
Immediate operands are integer literals that get encoded directly into the instructions.

They can be signed or unsigned and come in different sizes. The available sizes are:
- For unsigned immediates: 2 bits (`u2_imd`), 3 bits (`u3_imd`), 6 bits (`u6_imd`), 12 bits (`u12_imd`), 16 bits (`u16_imd`).
- For signed immediates: 9 bits (`s9_imd`), 10 bits (`s10_imd`), 12 bits (`s12_imd`), 19 bits (`s19_imd`), and 24 bits (`s24_imd`).

Some immediates (specifically addresses) are scaled by the data width. This means the immediate must be a multiple of the data width, and the assembler will shift the immediate down before insertion.

```
add r3, r2, 10
comp.eq c0, r3, +10
br +0x14
```

### Logic Immediate
```gramex
let logic_imd_level = "2" | "4" | "8" | "16" | "32" | "64";
let logic_imd = "logic_imd" "(" (logic_imd_level ",")? nb "," nb ")";
```
Logic immediates (`logic_imd`) are specific bitmask immediates used for logical operations.

They are created by the macro `logic_imd(level?, one_len, rot)`, which creates a continuous sequence of `1`s of length `one_len` starting from the least significant bit, then left-rotates it by `rot`.

The `logic_imd` macro also takes an optional pattern width (`level`, default is `64`) where the pattern gets repeated to fill a 64-bit word.

```
and r3, r1, logic_imd(16, 8) // r3 = r1 & 0xffff00
test.any c0, r3, logic_imd(48, 16) // test 16 bit overflow in r3
```

### Label Operands
```gramex
let label = ident;
```
Label operands encode their respective label as an immediate.

They can be used inside any immediate operand that can fit the label's offset.

```
br case_1
ld r1, [data]
```

### Address Operands
```gramex
let base_offset = gpr ("+" | "-") "="? nb;
let base_index = gpr | gpr "+" gpr ("shl" nb)?;
let address = "[" (imd | label | base_offset | base_index) "]";
let address_type = "offset" | "base_offset" | "base_index";
```
Address operands encode a memory address used in loads, stores, and branches.

Address operands are composed of an address formula enclosed in square brackets. These formulas can be: 
- **offset**: A scaled immediate or a label.
- **base + offset**: A general-purpose register plus a scaled immediate. If the sign is suffixed with `=`, the base register is updated with the computed address afterward.
- **base + index**: A general-purpose register plus an optionally shifted general-purpose register.

The offset and shift sizes are determined by each individual instruction. 

```
ld r1, [+0x10]
ld r1, [r2]
ld r1, [r2 + 0x10]
ld r1, [r2 + r3 shl 2]
```

## Constants
```gramex
let const = unsigned_nb_const | signed_nb_const | byte_arr_const | str_const;
```
A constant is a value that gets encoded into the binary at the current address.

A constant can be placed anywhere in the file and can span any length, however, it will be aligned as required.

The assembler inserts padding bytes after a constant to align the instruction that follows it.

A constant is composed of its type followed by its value. Constant types are:
- **`u8`, `u16`, `u32`, `u64`**: Unsigned numbers.
- **`i8`, `i16`, `i32`, `i64`**: Signed numbers.
- **`bytes`**: Byte arrays.
- **`str`**: Strings.

### Unsigned Number Constants
```gramex
let unsigned_nb_const = ("u8" | "u16" | "u32" | "u64") nb;
```
Unsigned number constants are unsigned 8, 16, 32, or 64-bit integer literals that are encoded into their binary form.

```
ff: u8 0xff
data: u32 0x12345678
```

### Signed Number Constants
```gramex
let signed_nb_const = ("i8" | "i16" | "i32" | "i64") ("-" | "+")? nb;
```
Signed number constants are signed 8, 16, 32, or 64-bit integer literals that are encoded into their binary form.

```
minus_one: i32 -1
```

### Byte Array Constants
```gramex
let byte_arr_const = "bytes" list<"\n"* nb, ",">;
```
Byte array constants are comma-separated arrays of bytes that get encoded in little-endian order.

Newlines can be used to separate the array across multiple lines, provided there is a comma at the end of each line.

```
data: bytes 
	0x01, 0x02, 0x03, 0x04,
	0x05, 0x06, 0x07, 0x08
```

### String Constants
```gramex
let escape_code = "\\" ("n" | "r" | "t" | "\"" | "\\" | "x" hex_dg hex_dg | "u{" hex_dg+ "}");
let str_const = "str" "\"" list<escape_code | !"\\"">* "\"";
```
String constants are UTF-8 encoded strings that get encoded into their binary form.

Strings are enclosed in double quotes and can contain the following escape sequences:
- `\n` newline.
- `\r` carriage return.
- `\t` horizontal tab.
- `\"` double quote.
- `\\` backslash.
- `\xhh` hex-encoded character.
- `\u{ccc}` unicode character code.

```
hello: str "hello world"
```

# Instruction Format
Instructions in MEP are a fixed 32 bits long, composed of multiple fields.

These fields are structured starting from the least significant bit and can represent opcodes, registers, immediates, and options/flags.

### Registers
Registers are encoded inside 5-bit fields called `gpr` based on their index.

| **Register** | **Encoding** | **Register** | **Encoding** | **Register** | **Encoding** | **Register** | **Encoding** |
| --------- | ------- | --------- | ------- | --------- | ------- | --------- | ------- |
| **`r0`**  | `00000` | **`r8`**  | `01000` | **`r16`** | `10100` | **`r24`** | `11000` |
| **`r1`**  | `00001` | **`r9`**  | `01001` | **`r17`** | `10101` | **`r25`** | `11001` |
| **`r2`**  | `00010` | **`r10`** | `01010` | **`r18`** | `10110` | **`r26`** | `11010` |
| **`r3`**  | `00011` | **`r11`** | `01011` | **`r19`** | `10111` | **`r27`** | `11011` |
| **`r4`**  | `00100` | **`r12`** | `01100` | **`r20`** | `11000` | **`r28`** | `11100` |
| **`r5`**  | `00101` | **`r13`** | `01101` | **`r21`** | `11001` | **`r29`** | `11101` |
| **`r6`**  | `00110` | **`r14`** | `01110` | **`r22`** | `11010` | **`r30`** | `11110` |
| **`r7`**  | `00111` | **`r15`** | `01111` | **`r23`** | `11011` | **`r31`** | `11111` |

`cond` is a 5-bit field that encodes a register holding a condition. It is similar to `gpr`, except `c0` replaces `r0`.

`gpr_pc` is a 5-bit field similar to `gpr`, except `pc` replaces `r0`.

### Immediates
Immediates are encoded directly inside instructions in fields of various sizes.

The immediate fields and their sizes are:
- For unsigned immediates: `u2_imd` (2 bits), `u3_imd` (3 bits), `u6_imd` (6 bits), `u12_imd` (12 bits).
- For signed immediates: `s9_imd` (9 bits), `s10_imd` (10 bits), `s12_imd` (12 bits), `s19_imd` (19 bits), and `s24_imd` (24 bits).

Signed immediates are encoded in two's complement. All immediates are contiguous inside the instruction except for `s9_imd`, where the sign bit is encoded separately.

Some immediates are scaled, the decoder will shift the immediate up to align with the data width before execution.

### Shifted Register
Some instructions take a shifted register operand, which is encoded in three fields: a `gpr` for the register, a `u6_imd` for the shift amount, and a `sh` field that specifies the shift type.

`sh` is a 2-bit field that can be one of:
- `00`: Logical left (`shl`)
- `01`: Logical right (`shr`)
- `10`: Arithmetic right (`sar`)
- `11`: Rotate left (`rol`)

### Logic Immediate
Logic immediates are encoded in three fields: `l0` (a 1-bit field that specifies the `level`), `ones` (a `u6_imd` that encodes `level` and `one_len`), and `rot` (a `u6_imd` that corresponds to its macro counterpart).

The concatenation of `l0` and `ones` gives the Huffman encoding of `level` and `one_len`:
| `l0` ~ `ones` | `level` | `one_len` |
| --------- | ---- | -------- |
| `0nnnnnn` | `64` | `nnnnnn` + `1` |
| `10nnnnn` | `32` | `nnnnn` + `1`  |
| `110nnnn` | `16` | `nnnn` + `1`   |
| `1110nnn` | `8`  | `nnn` + `1`    |
| `11110nn` | `4`  | `nn` + `1`     |
| `111110n` | `2`  | `n` + `1`      |

### Options
`cw` is a 2-bit field that modifies how a condition is written. It can be:
- **default (`00`)**: Overwrites the destination register with the computed condition.
- **`.and` (`01`)**: ANDs the computed condition with the destination register's condition.
- **`.or` (`11`)**: ORs the computed condition with the destination register's condition.

`ca` is a 1-bit field that modifies how a condition is written. It can be:
- **default (`0`)**: Overwrites the destination register with the computed condition.
- **`.and` (`1`)**: ANDs the computed condition with the destination register's condition.

`sz` is a 2-bit field that specifies the data width. It can be:
- `00`: 64-bit
- `01`: 32-bit
- `10`: 16-bit
- `11`: 8-bit

# Arithmetic Instructions
## Addition
### add (shifted register)
```
add dst:gpr, src1:gpr, src2:sh_reg
```
![add encoding](./ref-assets/add.svg)

Adds register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### add (immediate)
```
add dst:gpr, src1:gpr_pc, src2:u12_imd
```
![add imd encoding](./ref-assets/add_imd.svg)

Adds register/`pc` (`src1`) and immediate `src2`, and writes the result to the `dst` register.

### add.carry
```
add.carry dst:gpr, src1:gpr, src2:gpr, carry:cond
```
![add.carry encoding](./ref-assets/add_carry.svg)

Adds registers `src1` and `src2` with the carry flag from the `carry` register, writes the result to the `dst` register, and then updates the `carry` register.

### add triple
```
add dst:gpr, src1:gpr, src2:gpr, src3:gpr
```
![add3 encoding](./ref-assets/add3.svg)

Adds registers `src1`, `src2`, and `src3`, and writes the result to the `dst` register.

### cinc
```
cinc dst:gpr, cond:cond, src:gpr
```
![cinc encoding](./ref-assets/cinc.svg)

Writes `src` incremented by `1` to the `dst` register if the `cond` register is `true`, otherwise, moves `src` without modification.

## Subtract
### sub (shifted register)
```
sub dst:gpr, src1:gpr, src2:sh_reg
```
![sub encoding](./ref-assets/sub.svg)

Subtracts optionally shifted register `src2` from register `src1`, and writes the result to the `dst` register.

### sub reverse (shifted register)
```
sub dst:gpr, src1:sh_reg, src2:gpr
```
![sub rev encoding](./ref-assets/sub_rev.svg)

Subtracts register `src2` from optionally shifted register `src1`, and writes the result to the `dst` register.

### sub (immediate)
```
sub dst:gpr, src1:gpr, src2:u12_imd
```
![sub imd encoding](./ref-assets/sub_imd.svg)

Subtracts immediate `src2` from register `src1`, and writes the result to the `dst` register.

### sub reverse (immediate)
```
sub dst:gpr, src1:u12_imd, src2:gpr
```
![sub rev imd encoding](./ref-assets/sub_rev_imd.svg)

Subtracts register `src2` from immediate `src1`, and writes the result to the `dst` register.

### sub.borrow
```
sub.borrow dst:gpr, src1:gpr, src2:gpr, borrow:cond
```
![sub.borrow encoding](./ref-assets/sub_borrow.svg)

Subtracts register `src2` from `src1` with the borrow flag from the `borrow` register, writes the result to the `dst` register, and then updates the `borrow` register.

## Multiplication
### mult
```
mult dst:gpr, src1:gpr, src2:gpr
```
![mult encoding](./ref-assets/mult.svg)

Multiplies register `src1` by `src2`, and writes the result to the `dst` register.

It is an alias for [`mult.full dst, r0, src1, src2`](#multfull).

### mult.full
```
mult.full plow:gpr, phigh:gpr, src1:gpr, src2:gpr
```
![mult.full encoding](./ref-assets/mult_full.svg)

Multiplies register `src1` by `src2` to produce a full 128-bit product, then writes the low and high 64 bits to the `plow` and `phigh` registers respectively.

### madd
```
madd dst:gpr, src1:gpr, src2:gpr, src3:gpr
```
![madd encoding](./ref-assets/madd.svg)

Multiplies register `src1` by `src2`, adds register `src3` to the product, and writes the result to the `dst` register.

### msub
```
msub dst:gpr, src1:gpr, src2:gpr, src3:gpr
```
![msub encoding](./ref-assets/msub.svg)

Multiplies register `src1` by `src2`, subtracts the product from register `src3`, and writes the result to the `dst` register.

## Division
### div
```
div dst:gpr, src1:gpr, src2:gpr
```
![div encoding](./ref-assets/div.svg)

Divides register `src1` by `src2`, and writes the quotient to the `dst` register.

It is an alias for [`div.full dst, r0, src1, src2`](#divfull).

### rem
```
rem dst:gpr, src1:gpr, src2:gpr
```
![rem encoding](./ref-assets/rem.svg)

Computes the remainder of register `src1` divided by register `src2`, and writes the result to the `dst` register.

It is an alias for [`div.full r0, dst, src1, src2`](#divfull).

### div.full
```
div.full quo:gpr, rem:gpr, src1:gpr, src2:gpr
```
![div.full encoding](./ref-assets/div_full.svg)

Divides register `src1` by `src2`, then writes the quotient to the `quo` register and the remainder to the `rem` register.

### udiv
```
udiv dst:gpr, src1:gpr, src2:gpr
```
![udiv encoding](./ref-assets/udiv.svg)

Unsigned-divides register `src1` by `src2`, and writes the quotient to the `dst` register.

It is an alias for [`udiv.full dst, r0, src1, src2`](#udivfull).

### urem
```
urem dst:gpr, src1:gpr, src2:gpr
```
![urem encoding](./ref-assets/urem.svg)

Computes the unsigned remainder of register `src1` divided by register `src2`, and writes the result to the `dst` register.

It is an alias for [`udiv.full r0, dst, src1, src2`](#udivfull).

### udiv.full
```
udiv.full quo:gpr, rem:gpr, src1:gpr, src2:gpr
```
![udiv.full encoding](./ref-assets/udiv_full.svg)

Unsigned-divides register `src1` by `src2`, then writes the quotient to the `quo` register and the remainder to the `rem` register.

# Sign and Comparison Instructions
## Unary Operations
### abs
```
abs dst:gpr, src:gpr
```
![abs encoding](./ref-assets/abs.svg)

Computes the absolute value of register `src`, and writes the result to the `dst` register.

### neg
```
neg dst:gpr, src:sh_reg
```
![neg encoding](./ref-assets/neg.svg)

Negates an optionally shifted register `src`, and writes the result to the `dst` register.

It is an alias for [`sub dst, r0, src`](#sub-shifted-register).

### cneg
```
cneg dst:gpr, cond:cond, src:gpr
```
![cneg encoding](./ref-assets/cneg.svg)

Writes the negation of `src` to the `dst` register if the `cond` register is `true`, otherwise, moves `src` without modification.

### signed extend
```
se{.8, .16, .32} dst:gpr, src:gpr
```
![se encoding](./ref-assets/se.svg)

Sign-extends register `src` from 8, 16, or 32 bits, and writes the result to the `dst` register.

## Min / Max
### min
```
min dst:gpr, src1:gpr, src2:gpr
```
![min encoding](./ref-assets/min.svg)

Determines the signed minimum of registers `src1` and `src2`, and writes it to the `dst` register.

### max
```
max dst:gpr, src1:gpr, src2:gpr
```
![max encoding](./ref-assets/max.svg)

Determines the signed maximum of registers `src1` and `src2`, and writes it to the `dst` register.

### umin
```
umin dst:gpr, src1:gpr, src2:gpr
```
![umin encoding](./ref-assets/umin.svg)

Determines the unsigned minimum of registers `src1` and `src2`, and writes it to the `dst` register.

### umax
```
umax dst:gpr, src1:gpr, src2:gpr
```
![umax encoding](./ref-assets/umax.svg)

Determines the unsigned maximum of registers `src1` and `src2`, and writes it to the `dst` register.

## Equality Comparison
### comp.eq
```
comp.eq{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.eq encoding](./ref-assets/comp_eq.svg)

Determines if registers `src1` and `src2` are equal, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### comp.ne
```
comp.ne{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.ne encoding](./ref-assets/comp_ne.svg)

Determines if registers `src1` and `src2` are not equal, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### comp.eq immediate
```
comp.eq{_, .and} dst:cond, src1:gpr, src2:s12_imd
```
![comp.eq imd encoding](./ref-assets/comp_eq_imd.svg)

Determines if register `src1` is equal to immediate `src2`, and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

### comp.ne immediate
```
comp.ne{_, .and} dst:cond, src1:gpr, src2:s12_imd
```
![comp.ne imd encoding](./ref-assets/comp_ne_imd.svg)

Determines if register `src1` is not equal to immediate `src2`, and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

## Signed Comparison
### comp.gt
```
comp.gt{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.gt encoding](./ref-assets/comp_gt.svg)

Determines if register `src1` is strictly greater than register `src2`, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### comp.le
```
comp.le{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.le encoding](./ref-assets/comp_le.svg)

Determines if register `src1` is less than or equal to register `src2`, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### comp.gt immediate
```
comp.gt{_, .and} dst:cond, src1:gpr, src2:s12_imd
```
![comp.gt imd encoding](./ref-assets/comp_gt_imd.svg)

Determines if register `src1` is strictly greater than immediate `src2`, and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

## Unsigned Comparison
### ucomp.gt
```
ucomp.gt{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![ucomp.gt encoding](./ref-assets/ucomp_gt.svg)

Determines if register `src1` is strictly greater than register `src2` (unsigned), and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### ucomp.le
```
ucomp.le{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![ucomp.le encoding](./ref-assets/ucomp_le.svg)

Determines if register `src1` is less than or equal to register `src2` (unsigned), and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### ucomp.gt immediate
```
ucomp.gt{_, .and} dst:cond, src1:gpr, src2:u12_imd
```
![ucomp.gt imd encoding](./ref-assets/ucomp_gt_imd.svg)

Determines if register `src1` is strictly greater than immediate `src2` (unsigned), and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

# Logical Instructions
## Primary Operations
### not (shifted register)
```
not dst:gpr, src:sh_reg
```
![not encoding](./ref-assets/not.svg)

Inverts optionally shifted register `src`, and writes the result to the `dst` register.

It is an alias for [`xnor dst, r0, src`](#xnor-shifted-register).

### cnot
```
cnot dst:gpr, cond:cond, src:reg
```
![cnot encoding](./ref-assets/cnot.svg)

Writes the inverse of `src` to the `dst` register if the `cond` register is `true`, otherwise, moves `src` without modification.

### and (shifted register)
```
and dst:gpr, src1:gpr, src2:sh_reg
```
![and encoding](./ref-assets/and.svg)

Computes the bitwise AND of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### and (immediate)
```
and dst:gpr, src1:gpr, src2:logic_imd
```
![and imd encoding](./ref-assets/and_imd.svg)

Computes the bitwise AND of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

### or (shifted register)
```
or dst:gpr, src1:gpr, src2:sh_reg
```
![or encoding](./ref-assets/or.svg)

Computes the bitwise OR of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### or (immediate)
```
or dst:gpr, src1:gpr, src2:logic_imd
```
![or imd encoding](./ref-assets/or_imd.svg)

Computes the bitwise OR of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

### xor (shifted register)
```
xor dst:gpr, src1:gpr, src2:sh_reg
```
![xor encoding](./ref-assets/xor.svg)

Computes the bitwise exclusive OR of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### xor (immediate)
```
xor dst:gpr, src1:gpr, src2:logic_imd
```
![xor imd encoding](./ref-assets/xor_imd.svg)

Computes the bitwise exclusive OR of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

## Inverted Primary Operations
### nand (shifted register)
```
nand dst:gpr, src1:gpr, src2:sh_reg
```
![nand encoding](./ref-assets/nand.svg)

Computes the bitwise inverted AND of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### nand (immediate)
```
nand dst:gpr, src1:gpr, src2:logic_imd
```
![nand imd encoding](./ref-assets/nand_imd.svg)

Computes the bitwise inverted AND of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

### nor (shifted register)
```
nor dst:gpr, src1:gpr, src2:sh_reg
```
![nor encoding](./ref-assets/nor.svg)

Computes the bitwise inverted OR of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### nor (immediate)
```
nor dst:gpr, src1:gpr, src2:logic_imd
```
![nor imd encoding](./ref-assets/nor_imd.svg)

Computes the bitwise inverted OR of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

### xnor (shifted register)
```
xnor dst:gpr, src1:gpr, src2:sh_reg
```
![xnor encoding](./ref-assets/xnor.svg)

Computes the bitwise inverted exclusive OR of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### xnor (immediate)
```
xnor dst:gpr, src1:gpr, src2:logic_imd
```
![xnor imd encoding](./ref-assets/xnor_imd.svg)

Computes the bitwise inverted exclusive OR of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

## Secondary Operations
### bit clear (shifted register)
```
bcr dst:gpr, src1:gpr, src2:sh_reg
```
![bcr encoding](./ref-assets/bcr.svg)

Computes the bitwise AND of register `src1` and the inverse of optionally shifted register `src2`, and writes the result to the `dst` register.

### bit clear (immediate)
```
bcr dst:gpr, src1:gpr, src2:logic_imd
```
![bcr imd encoding](./ref-assets/bcr_imd.svg)

Computes the bitwise AND of register `src1` and the inverse of logic immediate `src2`, and writes the result to the `dst` register.

### imply (shifted register)
```
imply dst:gpr, src1:gpr, src2:sh_reg
```
![imply encoding](./ref-assets/imply.svg)

Computes the bitwise OR of the inverse of register `src1` and optionally shifted register `src2`, and writes the result to the `dst` register.

### imply (immediate)
```
imply dst:gpr, src1:gpr, src2:logic_imd
```
![imply imd encoding](./ref-assets/imply_imd.svg)

Computes the bitwise OR of the inverse of register `src1` and logic immediate `src2`, and writes the result to the `dst` register.

## Bit Test
### test.none
```
test.none dst:cond, src:gpr, mask:gpr
```
![test.none encoding](./ref-assets/test_none.svg)

Determines if all bits in register `src` specified by register `mask` are zero, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### test.none (immediate)
```
test.none dst:cond, src:gpr, mask:logic_imd
```
![test.none imd encoding](./ref-assets/test_none_imd.svg)

Determines if all bits in register `src` specified by logic immediate `mask` are zero, and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

### test.any
```
test.any dst:cond, src:gpr, mask:gpr
```
![test.any encoding](./ref-assets/test_any.svg)

Determines if any bit in register `src` specified by register `mask` is one, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### test.any (immediate)
```
test.any dst:cond, src:gpr, mask:logic_imd
```
![test.any imd encoding](./ref-assets/test_any_imd.svg)

Determines if any bit in register `src` specified by logic immediate `mask` is one, and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

### test.all
```
test.all dst:cond, src:gpr, mask:gpr
```
![test.all encoding](./ref-assets/test_all.svg)

Determines if all bits in register `src` specified by register `mask` are one, and writes the resulting condition to the `dst` register.

This instruction can optionally AND/OR the computed condition with `dst`.

### test.all (immediate)
```
test.all dst:cond, src:gpr, mask:logic_imd
```
![test.all imd encoding](./ref-assets/test_all_imd.svg)

Determines if all bits in register `src` specified by logic immediate `mask` are one, and writes the resulting condition to the `dst` register.

This instruction can optionally AND the computed condition with `dst`.

# Bit Manipulation Instructions
## Shifts
### shl
```
shl dst:gpr, src:gpr, amount:gpr
```
![shl encoding](./ref-assets/shl.svg)

Logically left-shifts register `src` by register `amount` modulo 64, and writes the result to the `dst` register.

This is an alias for [`fush dst, src, r0, amount`](#fush).

### shl (immediate)
```
shl dst:gpr, src:gpr, amount:u6_imd
```
![shl imd encoding](./ref-assets/shl_imd.svg)

Logically left-shifts register `src` by immediate `amount`, and writes the result to the `dst` register.

This is an alias for [`or dst, r0, src shl amount`](#or-shifted-register).

### shr
```
shr dst:gpr, src:gpr, amount:gpr
```
![shr encoding](./ref-assets/shr.svg)

Logically right-shifts register `src` by register `amount` modulo 64, and writes the result to the `dst` register.

### shr (immediate)
```
shr dst:gpr, src:gpr, amount:u6_imd
```
![shr imd encoding](./ref-assets/shr_imd.svg)

Logically right-shifts register `src` by immediate `amount`, and writes the result to the `dst` register.

This is an alias for [`or dst, r0, src shr amount`](#or-shifted-register).

### sar
```
sar dst:gpr, src:gpr, amount:gpr
```
![sar encoding](./ref-assets/sar.svg)

Arithmetically right-shifts register `src` by register `amount` modulo 64, and writes the result to the `dst` register.

### sar (immediate)
```
sar dst:gpr, src:gpr, amount:u6_imd
```
![sar imd encoding](./ref-assets/sar_imd.svg)

Arithmetically right-shifts register `src` by immediate `amount`, and writes the result to the `dst` register.

This is an alias for [`or dst, r0, src sar amount`](#or-shifted-register).

### rol
```
rol dst:gpr, src:gpr, amount:gpr
```
![rol encoding](./ref-assets/rol.svg)

Rotates register `src` left by register `amount` modulo 64, and writes the result to the `dst` register.

This is an alias for [`fush dst, src, src, amount`](#fush).

### rol (immediate)
```
rol dst:gpr, src:gpr, amount:u6_imd
```
![rol imd encoding](./ref-assets/rol_imd.svg)

Rotates register `src` left by immediate `amount`, and writes the result to the `dst` register.

This is an alias for [`or dst, r0, src rol amount`](#or-shifted-register).

### fush
```
fush dst:gpr, src:gpr, carry:gpr, amount:gpr
```
![fush encoding](./ref-assets/fush.svg)

Funnel left-shifts register `src` by register `amount` modulo 64, taking the carry from the `carry` register, and writes the result to the `dst` register.

## Bitfield
### bfext
```
bfext dst:gpr, src:gpr, offset:gpr, width:gpr
```
![bfext encoding](./ref-assets/bfext.svg)

Extracts a bitfield of (register `width` modulo `64` + `1`) bits starting at bit position (register `offset` modulo 64) from register `src`, and writes the result to the `dst` register.

### bfext (immediate)
```
bfext dst:gpr, src:gpr, offset:u6_imd, width:u6_imd
```
![bfext imd encoding](./ref-assets/bfext_imd.svg)

Extracts a bitfield of (immediate `width` + `1`) bits starting at the bit position specified by immediate `offset` from register `src`, and writes the result to the `dst` register.

### bfins
```
bfins dst:gpr, src:gpr, offset:gpr, width:gpr 
```
![bfins encoding](./ref-assets/bfins.svg)

Extracts a bitfield of (register `width` modulo `64` + `1`) bits starting from bit `0` from register `src`, and inserts it into register `dst` at the bit position (register `offset` modulo 64) without affecting the other bits.

### bfins (immediate)
```
bfins dst:gpr, src:gpr, offset:u6_imd, width:u6_imd
```
![bfins imd encoding](./ref-assets/bfins_imd.svg)

Extracts a bitfield of (immediate `width` modulo `64` + `1`) bits starting from bit `0` from register `src`, and inserts it into register `dst` at the bit position specified by immediate `offset` without affecting the other bits.

## Bit Counting
### cnt
```
cnt dst:gpr, src:gpr
```
![cnt encoding](./ref-assets/cnt.svg)

Counts the number of `1` bits in register `src`, and writes the result to the `dst` register.

### cntz
```
cntz dst:gpr, src:gpr
```
![cntz encoding](./ref-assets/cntz.svg)

Counts the number of `0` bits in register `src`, and writes the result to the `dst` register.

### clz
```
clz dst:gpr, src:gpr
```
![clz encoding](./ref-assets/clz.svg)

Counts the number of leading (most significant) `0` bits in register `src`, and writes the result to the `dst` register.

### clo
```
clo dst:gpr, src:gpr
```
![clo encoding](./ref-assets/clo.svg)

Counts the number of leading (most significant) `1` bits in register `src`, and writes the result to the `dst` register.

### ctz
```
ctz dst:gpr, src:gpr
```
![ctz encoding](./ref-assets/ctz.svg)

Counts the number of trailing (least significant) `0` bits in register `src`, and writes the result to the `dst` register.

### cto
```
cto dst:gpr, src:gpr
```
![cto encoding](./ref-assets/cto.svg)

Counts the number of trailing (least significant) `1` bits in register `src`, and writes the result to the `dst` register.

### cls
```
cls dst:gpr, src:gpr
```
![cls encoding](./ref-assets/cls.svg)

Counts the number of leading (most significant) bits with the same value as bit `63` in register `src`, and writes the result to the `dst` register.

## Reversion
### rev
```
rev dst:gpr, src:gpr
```
![rev encoding](./ref-assets/rev.svg)

Reverses the bits in register `src`, and writes the result to the `dst` register.

### reverse parts
```
rev{.8, .16, .32} dst:gpr, src:gpr
```
![rev parts encoding](./ref-assets/rev_parts.svg)

Reverses the order of 8, 16, or 32-bit parts in register `src`, and writes the result to the `dst` register.

# Data Movement Instructions
## Move
### mov (register)
```
mov dst:gpr, src:gpr
```
![mov encoding](./ref-assets/mov.svg)

Moves register `src` to the `dst` register.

This is an alias for [`or dst, src, r0`](#or-shifted-register).

### mov (immediate)
```
mov dst:gpr, src:u16_imd, sh?: u2_imd
```
![mov imd encoding](./ref-assets/mov_imd.svg)

Writes immediate `src` left-shifted by (immediate `sh` * `16`) to the `dst` register.

### mov (logic immediate)
```
mov dst:gpr, src:logic_imd
```
![mov logic imd encoding](./ref-assets/mov_logic_imd.svg)

Writes the logic immediate `src` to the `dst` register.

This is an alias for [`or dst, r0, src`](#or-immediate).

### mov (negative immediate)
```
move dst:gpr, -src:u12_imd
```
![mov neg imd encoding](./ref-assets/mov_neg_imd.svg)

Writes the negative immediate `src` to the `dst` register.

This is an alias for [`sub dst, r0, src`](#sub-immediate).

### mov.keep
```
mov.keep dst:gpr, src:u16_imd, sh:u2_imd
```
![mov.keep encoding](./ref-assets/mov_keep.svg)

Writes immediate `src` into the 16-bit part of the `dst` register specified by immediate `sh` without affecting the other bits.

## Conditional Select
### sel
```
sel dst:gpr, cond:cond, src1:gpr, src2:gpr
```
![sel encoding](./ref-assets/sel.svg)

Writes the value of register `src1` to the `dst` register if the `cond` register is `true`, otherwise, writes the value of register `src2`.

### sel (immediate)
```
sel dst:gpr, cond:cond, src1:gpr, src2:s9_imd
```
![sel imd encoding](./ref-assets/sel_imd.svg)

Writes the value of register `src1` to the `dst` register if the `cond` register is `true`, otherwise, writes the immediate `src2`.

`src2` is an `s9_imd` where the sign bit is encoded in the `s` field.

## Memory Access (Offset)
### ld (offset)
```
ld{_, .32, .16, .8} dst:gpr, loc:offset
```
![ld offset encoding](./ref-assets/ld_offset.svg)

Loads the 64, 32, 16, or 8-bit value from memory at address `loc`, and writes the result to the `dst` register.

The address is computed as an `s19_imd` relative `offset` scaled by the data width.

## Memory Access (Base + Offset)
![mem access base offset encoding](./ref-assets/mem_base_offset.svg)

The address is computed from a `base` register plus an `s12_imd` `offset` scaled by the data width.

If the `w` field is `1`, the `base` register is updated with the computed address.

### ld (base + offset)
```
ld{_, .32, .16, .8} dst:gpr, loc:base_offset
```
![ld base offset encoding](./ref-assets/ld_base_offset.svg)

Loads the 64, 32, 16, or 8-bit value from memory at address `loc`, and writes the result to the `dst` register.

### st (base + offset)
```
st{_, .32, .16, .8} src:gpr, loc:base_offset
```
![st base offset encoding](./ref-assets/st_base_offset.svg)

Stores the 64, 32, 16, or 8-bit value from the `src` register into memory at address `loc`.

### ld.s (base + offset)
```
ld{.s32, .s16, .s8} dst:gpr, loc:base_offset
```
![ld.s base offset encoding](./ref-assets/ld_s_base_offset.svg)

Loads the 32, 16, or 8-bit value from memory at address `loc`, and writes the sign-extended result to the `dst` register.

## Memory Access (Base + Index)
![mem access base index encoding](./ref-assets/mem_base_index.svg)

The address is computed from a `base` register plus an `index` register.

If the `s` field is `1`, the `index` register is scaled by the data width.

### ld (base + index)
```
ld{_, .32, .16, .8} dst:gpr, loc:base_index
```
![ld base index encoding](./ref-assets/ld_base_index.svg)

Loads the 64, 32, 16, or 8-bit value from memory at address `loc`, and writes the result to the `dst` register.

### st (base + index)
```
st{_, .32, .16, .8} src:gpr, loc:base_index
```
![st base index encoding](./ref-assets/st_base_index.svg)

Stores the 64, 32, 16, or 8-bit value from the `src` register into memory at address `loc`.

### ld.s (base + index)
```
ld{.s32, .s16, .s8} dst:gpr, loc:base_index
```
![ld.s base index encoding](./ref-assets/ld_s_base_index.svg)

Loads the 32, 16, or 8-bit value from memory at address `loc`, and writes the sign-extended result to the `dst` register.

# Control Flow Instructions
## Branches
### br
```
br offset:s24_imd
```
![br encoding](./ref-assets/br.svg)

Performs an unconditional branch by a `pc`-relative `offset` scaled by 4 bytes.

### br (link)
```
br link:gpr, offset:s19_imd
```
![br link encoding](./ref-assets/br_link.svg)

Stores the address of the next instruction in the `link` register, then performs an unconditional branch by a `pc`-relative `offset` scaled by 4 bytes.

### br (cond)
```
br{.true, .false} cond:cond, offset:s19_imd
```
![br.cond encoding](./ref-assets/br_cond.svg)

Branches by a `pc`-relative `offset` scaled by 4 bytes, based on the `cond` register.

If the `c` field is `1`, the branch is taken if `cond` is `true`, otherwise, it takes the inverse.

## Jumps
### jmp (base + index)
```
jmp link?:gpr, loc:base_index
```
![jmp index encoding](./ref-assets/jmp_index.svg)

Stores the address of the next instruction in the `link` register, then performs an unconditional branch to address `loc`.

`loc` is computed from a `base` register plus an `index` register scaled by 4 bytes and further shifted by the `sh` field.

### jmp (base + offset)
```
jmp link?:gpr, loc:base_offset
```
![jmp offset encoding](./ref-assets/jmp_offset.svg)

Stores the address of the next instruction in the `link` register, then performs an unconditional branch to address `loc`.

`loc` is computed from a `base` register plus an `s10_imd` `offset` scaled by 4 bytes.

### halt
```
halt
```
![halt encoding](./ref-assets/halt.svg)

Halts and ends execution without catching on fire.

# Index by Encoding
![root](./ref-assets/root_enc.svg)

| `pgrp` | Primary Group |
| ------ | --- | 
| `0001` | [`dpr`](#data-processing-register) |
| `0010` | [`dpi`](#data-processing-immediate) |
| `0011` | [`mem`](#memory-1) |
| `0100` | [`branch`](#branch) |

## Data Processing Register
![dpr root](./ref-assets/dpr_enc.svg)

| `grp` | Group |
| ------ | --- |
| `0000` | [**3 regs**](#3-regs) |
| `0001` | [**2 regs**](#2-regs) |
| `0010` | [**4 regs**](#4-regs) |
| `1xxx` | [**shift**](#shift) |

### 3 regs
#### Format 1
![3 regs format 1](./ref-assets/3_regs_f1_enc.svg)

#### Format 2
![3 regs format 2](./ref-assets/3_regs_f2_enc.svg)

#### Format 3
![3 regs format 3](./ref-assets/3_regs_f3_enc.svg)

| `sub_grp` | `op` | Format | Instruction |
| ----- | --- | --- | --- |
| `0000` | `00000` | 1 | [`shr`](#shr) |
| `0000` | `00001` | 1 | [`sar`](#sar) |
| `0000` | `00010` | 1 | [`min`](#min) |
| `0000` | `00011` | 1 | [`max`](#max) |
| `0000` | `00100` | 1 | [`umin`](#umin) |
| `0000` | `00101` | 1 | [`umax`](#umax) |
| `0000` | `00110` | 2 | [`cnot`](#cnot) |
| `0000` | `00111` | 2 | [`cinc`](#cinc) |
| `0000` | `01000` | 2 | [`cneg`](#cneg) |
| `0001` | `000` | 3 | [`comp.eq`](#compeq) |
| `0001` | `001` | 3 | [`comp.ne`](#compne) |
| `0001` | `010` | 3 | [`comp.gt`](#compgt) |
| `0001` | `011` | 3 | [`comp.le`](#comple) |
| `0001` | `100` | 3 | [`ucomp.gt`](#ucompgt) |
| `0001` | `101` | 3 | [`ucomp.le`](#ucomple) |
| `0010` | `000` | 3 | [`test.none`](#testnone) |
| `0010` | `001` | 3 | [`test.any`](#testany) |
| `0010` | `010` | 3 | [`test.all`](#testall) |

| `cw` | Modifier |
| --- | --- |
| `00` | default |
| `01` | AND |
| `11` | OR |

### 2 regs
![2 regs](./ref-assets/2_regs_enc.svg)

| `op` | Instruction |
| ---- | --- |
| `000000` | [`cnt`](#cnt) |
| `000001` | [`cntz`](#cntz) |
| `000010` | [`abs`](#abs) |
| `000011` | [`cls`](#cls) |
| `000100` | [`clz`](#clz) |
| `000101` | [`clo`](#clo) |
| `000110` | [`ctz`](#ctz) |
| `000111` | [`cto`](#cto) |
| `001000` | [`rev`](#rev) |
| `001001` | [`rev.32`](#reverse-parts) |
| `001010` | [`rev.16`](#reverse-parts) |
| `001011` | [`rev.8`](#reverse-parts) |
| `001101` | [`se.32`](#signed-extend) |
| `001110` | [`se.16`](#signed-extend) |
| `001111` | [`se.8`](#signed-extend) |

### 4 regs
#### Format 1
![4 regs format 1](./ref-assets/4_regs_f1_enc.svg)

#### Format 2
![4 regs format 2](./ref-assets/4_regs_f2_enc.svg)

#### Format 3
![4 regs format 3](./ref-assets/4_regs_f3_enc.svg)

| `op` | Format | Instruction |
| ---- | --- | --- |
| `0000` | 2 | [`add.carry`](#addcarry) |
| `0001` | 2 | [`sub.borrow`](#subborrow) |
| `0010` | 1 | [`add` triple](#add-triple) |
| `0011` | 1 | [`madd`](#madd) |
| `0100` | 1 | [`msub`](#msub) |
| `0101` | 1 | [`mul.full`](#multfull) |
| `0110` | 1 | [`div.full`](#divfull) |
| `0111` | 1 | [`udiv.full`](#udivfull) |
| `1000` | 1 | [`fush`](#fush) |
| `1001` | 1 | [`bfext`](#bfext) |
| `1010` | 1 | [`bfins`](#bfins) |
| `1011` | 3 | [`sel`](#sel) |

### Shift
![shift](./ref-assets/shift_enc.svg)

| `o1` | `op2` | Instruction |
| --- | --- | --- |
| `0` | `000` | [`add`](#add-shifted-register) |
| `0` | `001` | [`sub`](#sub-shifted-register) |
| `0` | `010` | [`sub` reverse](#sub-reverse-shifted-register) |
| `1` | `000` | [`and`](#and-shifted-register) |
| `1` | `001` | [`or`](#or-shifted-register) |
| `1` | `010` | [`xor`](#xor-shifted-register) |
| `1` | `011` | [`imply`](#imply-shifted-register) |
| `1` | `100` | [`nand`](#nand-shifted-register) |
| `1` | `101` | [`nor`](#nor-shifted-register) |
| `1` | `110` | [`xnor`](#xnor-shifted-register) |
| `1` | `111` | [`bcr`](#bit-clear-shifted-register) |

| `sh` | Shift |
| --- | --- |
| `00` | `shl` |
| `01` | `shr` |
| `10` | `sar` |
| `11` | `rol` |

## Data Processing Immediate
![dpi encoding](./ref-assets/dpi_enc.svg)

| `grp` | Group |
| ----- | ------|
| `00xx` | [Logic](#logic) |
| `0100` | [Arith](#arith) |
| `0101` \| `0110` | [Comp](#comp) |
| `0111` \| `1000` \| `1001` | [Bit Test](#bit-test-1) |
| `1010` | [Move Wide](#move-wide) |
| `1011` | [`sel`](#sel-immediate) |
| `1100` | [Bitfield](#bitfield-1) |

### Logic
![dpi logic encoding](./ref-assets/dpi_logic_enc.svg)

| `op1` | `o2` | Instruction |
| ------| ---- | --- |
| `00` | `0` | [`and`](#and-immediate) |
| `00` | `1` | [`or`](#or-immediate) |
| `01` | `0` | [`xor`](#xor-immediate) |
| `01` | `1` | [`imply`](#imply-immediate) |
| `10` | `0` | [`nand`](#nand-immediate) |
| `10` | `1` | [`nor`](#nor-immediate) |
| `11` | `0` | [`xor`](#xor-immediate) |
| `11` | `1` | [`bcr`](#bcr-immediate) |

### Arith
![dpi arith encoding](./ref-assets/dpi_arith_enc.svg)

| `op` | Instruction |
| ---- | --- |
| `00` | [`add`](#add-immediate) |
| `01` | [`sub`](#sub-immediate) |
| `10` | [`sub` reverse](#sub-reverse-immediate) |

### Comp
![dpi comp encoding](./ref-assets/dpi_comp_enc.svg)

| `grp` | `op` | Instruction | `src2_imd` |
| ----- | ---- | --- | --- |
| `0101` | `0` | [`comp.eq`](#compeq-immediate) | `s12_imd` |
| `0101` | `1` | [`comp.ne`](#compne-immediate) | `s12_imd` |
| `0110` | `0` | [`comp.gt`](#compgt-immediate) | `s12_imd` |
| `0110` | `1` | [`ucomp.gt`](#ucompgt-immediate) | `u12_imd` |

| `ca` | Modifier |
| --- | --- |
| `0` | default |
| `1` | AND |

### Bit Test
![dpi bit test encoding](./ref-assets/dpi_bit_test_enc.svg)

| `grp` | Instruction |
| ----- | --- |
| `0111` | [`test.none`](#testnone-immediate) |
| `1000` | [`test.any`](#testany-immediate) |
| `1001` | [`test.all`](#testall-immediate) |

| `ca` | Modifier |
| --- | --- |
| `0` | default |
| `1` | AND |

### Move Wide
![dpi move wide encoding](./ref-assets/dpi_move_wide_enc.svg)

| `op` | Instruction |
| ---- | --- |
| `0` | [`mov` wide](#mov-immediate) |
| `1` | [`mov.keep`](#movkeep) |

### Bitfield
![dpi bitfield encoding](./ref-assets/dpi_bitfield_enc.svg)

| `op` | Instruction |
| ---- | --- |
| `00` | [`bfext`](#bfext-immediate) |
| `01` | [`bfins`](#bfins-immediate) |

## Memory
![mem encoding](./ref-assets/mem_enc.svg)

| `amod` | Addressing Mode |
| ------ | --- |
| `0x` | [base + offset](#base--offset) |
| `10` | [base + index](#base--index) |
| `11` | [`ld` offset](#ld-offset) |

### Base + Offset
![mem base offset encoding](./ref-assets/mem_base_offset_enc.svg)

| `op` | Instruction |
| ---- | --- |
| `00` | [`ld`](#ld-base--offset) |
| `01` | [`st`](#st-base--offset) |
| `10` | [`ld.s`](#lds-base--offset) |

| `w` | Modifier |
| --- | --- |
| `0` | default |
| `1` | write back |

### Base + Index
![mem base index encoding](./ref-assets/mem_base_index_enc.svg)

| `op` | Instruction |
| ---- | --- |
| `00` | [`ld`](#ld-base--index) |
| `01` | [`st`](#st-base--index) |
| `10` | [`ld.s`](#lds-base--index) |

| `s` | Modifier |
| --- | --- |
| `0` | no scale |
| `1` | scale |

## Branch
![br encoding](./ref-assets/br_enc.svg)

| `op1` | `op2` | Instruction |
| ----- | ---- | --- |
| `0000` | `_` | [`br`](#br) |
| `0001` | `_` | [`br` link](#br-link) |
| `001x` | `_` | [`br.cond`](#br-cond) |
| `0100` | `0000` | [`jmp` base + index](#jmp-base--index) |
| `0100` | `0001` | [`jmp` base + offset](#jmp-base--offset) |