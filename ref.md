# MEP
MEP (Mapped Expressions Processor) is an advance 64 bit risc cpu implemented in unimap.

mep is little endian von neumann cpu, it has the features of a regular risc cpu, however it has a wide range of powerfull operations and compound instructions typically found in modern isas.

this document serve as the architectural reference for mep, describing its assembly language, and detailing its instruction encoding.

# execution model
## data types
mep process binary data in littlle endian order, it access bytes and bits from least significant to most significant.

its native word size is 64 bits, the size of the general purpose registers, memory addresses, and the data width of most common operations.

mep supports unsigned and signed twos complement integers at 8, 16, 32 and 64 bits sizes.

## regesters
the majority of instructions operate on general purpose registers (`gpr`). mep provides 31 64 bit general purpose registers named `R1` through `R31`.

these registers can be used freely for any purpose and are accessable to all instructions.

`R0` (the zero register) is a general purpose register that is hardcoded to 0, any read from it will resolve to 0, and all writes are discarded.

`PC` (program counter) is an internal 64 bit register that holds the momery address of the current executing instruction, it is not accessible or modifiable by normal instruction, it is only accessable through special instructions.

### condition registers
mep doesnt have a traditional flagss register, instead all gprs can be used as condition registers.

when evaluated as a condition, zero value is `false`, while non-zero values are `true`.

`R0` used as a condition register redirects to a 1 bit `C0` register, it can hold a condition without overwriting another register.

## memory
memory in mep is byte addressable with 64 bit address space.

all fundamental data types is stored in memory in little endian order and must be aligned to their respective size.

mep is a load store architecture, all memory accesses are performed only by `ld` and `st` instructions.

# assembly language
the assembly language is a human readable form of mep executable code.
## basic syntax
this section uses the [gramex meta language](https://docs.rs/gramex/latest/gramex/docs/gram_ref/index.html), and except the file to be valid utf-8.

whitespace are insignificant, they are only used to separate tokens, the whitespace characters are: space ` `, horizontal tab `\t`, and carriage return `\r`.

line feed `\n` is used as separater between instructions.

#### comments
```gramex
let comment = "//" !"\n"* "\n"? | "/*" !"*/"* "*/";
```
mep assembly supports line and block comments with their respective c style syntax.

they are ignored by the parser, and dont provide any semantic meaning.

#### identifiers
```gramex
let ident = ("a".."z" | "A".."Z") ("a".."z" | "A".."Z" | "0".."9" | "_")*;
```
identifiers are used as name for instructions, registers, labels and constants.

#### numbers
```gramex
let hex_dg = "0".."9" | "a".."f" | "A".."F";
let nb = "0b" ("0" | "1") ("_"? ("0" | "1"))* | "0x" hex_dg ("_"? hex_dg)* | "0".."9" ("_"? "0".."9")*;
```
numbers are unsigned integers used as offsets, immediate and constants.

they can be written in decimal, binary or hexadecimal, with optional `_` as separator for readability.

```
123 0b1001_0101 0xff_AA
```

## top level structure
```gramex
let file = list<label_decl? (inst | const), "\n"+>
```
an assembly file consist of instructions and constants separated by new lines, each instruction or constant can be prefixed with a label.

the assembler encode each instruction and constant into its binary form, then lay them after each other starting at address 0.

```
start: 
	ld.u8 r1, [one]
	mov r2, 2
	add r3, r1, r2

one: u8 0x01;
```

### labels
```gramex
let label_decl = ident ":"
```
labels are used to reference an instruction or constant by name inside immediates and offsets.

they resolve to the offset of the referenced item from the current instruction.

```
data: u32 0x12345678
loop_start: 
	ld.u32 r1, [data]
```

## instructions
```gramex
let mnemonic = list<ident, ".">;
let inst = mnemonic list<oprand, ",">?;
```
an instruction consists of its mnemonic followed by its operands.

instruction mnemonics can be upper or lower case, they are composed of a `.` separated list of identifiers.

each instruction can have 0 or more operands separated by `,`.

```
add r3, r1, r2
```

### instruction decleration
```gramex
let inst_decl = mnemonic ("{" list<"_" | "." ident, ","> "}")? list<ident "?"? ":" oprand_type, ",">?;
```
instruction declerations found in this document are composed of the intruction mnemonic followed by its operands decleration separated by `,`.

an instruction decleration can declare multiple sub / modified instructions by suffixing the mnemonic with a variant decleration.

the variant decleration is a comma separated list of variant suffixes enclosed inside curly brackets, a suffix can be a `_` (default variant) or a `.` followed by an identifier.

an oprand decleration consist of the oprand identifier followed by its type, an oprand is optional is its identifier is suffixed with `?`.

```
add dst:gpr, src1:gpr, src2:u12_imd
comp.eq{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```

### opreands
```gramex
let oprand = reg | sh_reg | imd | label | address;
let oprand_type = reg_type | "sh_reg" | imd_type | address_type; 
```
an oprand encodes a storage location, values or options the instruction take / perform on.

oprands come in different forms, they can registers, immediates, memory locations and labels.

```
add r3, r1, r2 shl 3
comp.eq c0, r3, +10
br.true c0, case_1
ld r4, [r1 += 0x10]
```

### register oprands
```gramex
let gpr = ("r" | "R")("0".."9" | ("1" | "2") "0".."9" | "30" | "31");
let pc = "pc" | "PC";
let c0 = "c0" | "C0";
let reg = gpr | pc | c0;
let reg_type = "gpr" | "cond" | "gpr_pc";
```
register oprands are the most common oprand types, they come in different types:
- `gpr`: any general purpose registers.
- `cond`: register holding a condition, `C0` and any general purpose registers except `R0`.
- `gpr_pc`: `PC` and any general purpose registers except `R0`.

register names can be upper or lower.

```
add r3, R1, r2
comp.eq c0, r1, r2
add r3, PC, +4
```

### shifted register oprands
```gramex
let sh_reg = gpr | gpr ("shl" | "shr" | "sar" | "rol" | "SHL" | "SHR" | "SAR" | "ROL) nb; 
```
shifted register oprands are general purpose registers shifted by a constant encoded in a 6 bit immediate.

the supported shifts are logical left (`shl`), logical right (`shr`), arithmetic right (`sar`), and rotate left (`rol`).

the shift part can be omitted, resulting in `shl 0`.

```
add r3, r1, r2 shl 4
or r4, r1, r3 rol 31
```

### immediate oprands
```gramex
let imd = ("+" | "-")? nb | logic_imd;
let imd_type = "u6_imd" | "u12_imd" | "s9_imd" | "s10_imd" | "s12_imd" | "s19_imd" | "s24_imd" | "logic_imd";
```
immediate oprands are integer literals that get encoded directly into the instructions.

they can be unsigned or unsigned and of different sizes, the available sizes are:
- for unsigned immediates: 6 bits (`u6_imd`) and 12 bits (`u12_imd`).
- for signed immediates: 9 bits (`s9_imd`), 10 bits (`s10_imd`), 12 bits (`s12_imd`), 19 bits (`s19_imd`) and 24 bits (`s24_imd`).

the sign is forbidden for unsigned immediates and required for signed ones.

some immediates (specifically addresses) are scalled by the data width, meaning that the immediate must be multiple of the data width, then the assembler will shift the immediate down before insertion.

```
add r3, r2, 10
comp.eq c0, r3, +10
br +0x14
```

### logic immediate
```gramex
let logic_imd_level = "2" | "4" | "8" | "16" | "32" | "64";
let logic_imd = "logic_imd" "(" (logic_imd_level ",")? nb "," nb ")";
```
logic immediate (`logic_imd`) are specific bitmask immediates for the logical operations.

they are created by the macro `logic_imd(level?, one_len, rot)` that creates a continuous sequence of `1` of len `one_len` from low, then left rotate it by `rot`.

`logic_imd` macro also takes an optional pattern width (`level`, default is `64`) where the pattern get repeated to be fill a 64 bit word.

```
and r3, r1, logic_imd(16, 8) // r3 = r1 & 0xffff00
test.any c0, r3, logic_imd(48, 16) // test 16 bit overflow in r3
```

### label oprands
```gramex
let label = ident;
```
label oprands encode their respective label as an immediate.

they can be used inside any immediate oprand that can fit the label offset.

```
br case_1
ld r1, [data]
```

### address oprands
```gramex
let base_offset = gpr ("+" | "-") "="? nb;
let base_index = gpr | gpr "+" gpr (("shl" | "SHL") nb)?;
let address = "[" (imd | label | base_offset | base_index) "]";
let address_type = "offset" | "base_offset" | "base_index";
```
address oprands encodes a memory address used in loads, stores and branches.

address oprands are composed of an address formula enclosed around square brackets, these formulas can be: 
- **offset**: a scaled immediate or a label.
- **base + offset**: a general purpose register plus a scaled immediate, if the sign is suffixed with `=`, the base register is updated with the computed address afterwards.
- **base + index**: a general purpose register plus an optionally shifted general purpose register.

the offset and shift sizes are determined by each individual instruction. 

```
ld r1, [+0x10]
ld r1, [r2]
ld r1, [r2 + 0x10]
ld r1, [r2 + r3 shl 2]
```

## constants
```gramex
let const = unsigned_nb_const | signed_nb_const | byte_arr_const | str_const;
```
a constant is a value that gets encoded into the binary at the current.

a constant can be put anywhere in the file, and can span any length, however it will be aligned like required.

the assembler will insert padding bytes after a constant to align the next instruction after it.

a constant is composed of its type followed by its value, constant types are:
- **`u8`, `u16`, `u32`, `u64`**: unsigned number.
- **`i8`, `i16`, `i32`, `i64`**: signed number.
- **`bytes`**: byte array.
- **`str`**: string.

### unsigned numbers constants
```gramex
let unsigned_nb_const = ("u8" | "u16" | "u32" | "u64") nb;
```
unsigned numbers constants are unsigned 8, 16, 32 or 64 bits integer literals that get encoded into their binary form.

```
ff: u8 0xff
data: u32 0x12345678
```

### signed numbers constants
```gramex
let signed_nb_const = ("i8" | "i16" | "i32" | "i64") ("-" | "+")? nb;
```
signed numbers constants are signed 8, 16, 32 or 64 bits integer literals that get encoded into their binary form.

```
minus_one: i32 -1
```

### byte array constants
```gramex
let byte_arr_const = "bytes" list<"\n"* nb, ",">;
```
byte array constants are comma separated array of bytes that get encoded in little endian order.

a newline can be used to separate the array into multiple lines, required to have a comma at the end of each line.

```
data: bytes 
	0x01, 0x02, 0x03, 0x04,
	0x05, 0x06, 0x07, 0x08
```

### string constants
```gramex
let escape_code = "\\" ("n" | "r" | "t" | "\"" | "\\" | "x" hex_dg hex_dg | "u{" hex_dg+ "}");
let str_const = "str" "\"" list<escape_code | !"\\"">* "\"";
```
string constants are utf-8 encoded strings that get encoded into their binary form.

strings are double quoted and can have the following escape sequences:
- `\n` newline.
- `\r` carriage return.
- `\t` horizontal tab.
- `\"` double quote.
- `\\` backslash.
- `\xhh` hex encoded character.
- `\u{ccc}` unicode character code.

```
hello: str "hello world"
```

# instruction format
instructions in mep are fixed 32 bit long, composed into multiple fields.

these fields are structured from the least significant bit, and can be opcodes, registers, immediate and options / flags.

### registers
registers are encoded inside 5 bit fields called `gpr` based on their index.

| **register** | **encoding** | **register** | **encoding** | **register** | **encoding** | **register** | **encoding** |
| --------- | ------- | --------- | ------- | --------- | ------- | --------- | ------- |
| **`R0`**  | `00000` | **`R8`**  | `01000` | **`R16`** | `10100` | **`R24`** | `11000` |
| **`R1`**  | `00001` | **`R9`**  | `01001` | **`R17`** | `10101` | **`R25`** | `11001` |
| **`R2`**  | `00010` | **`R10`** | `01010` | **`R18`** | `10110` | **`R26`** | `11010` |
| **`R3`**  | `00011` | **`R11`** | `01011` | **`R19`** | `10111` | **`R27`** | `11011` |
| **`R4`**  | `00100` | **`R12`** | `01100` | **`R20`** | `11000` | **`R28`** | `11100` |
| **`R5`**  | `00101` | **`R13`** | `01101` | **`R21`** | `11001` | **`R29`** | `11101` |
| **`R6`**  | `00110` | **`R14`** | `01110` | **`R22`** | `11010` | **`R30`** | `11110` |
| **`R7`**  | `00111` | **`R15`** | `01111` | **`R23`** | `11011` | **`R31`** | `11111` |

`cond` is a 5 bit field that encodes a register holding a condition, it is simmilar to `gpr` except `C0` replaces `R0`.

`gpr_pc` is a 5 bit field similar to `gpr` execept `PC` replaces `R0`.

### immediates
immediates are encoded directly inside instructions in fields of various sizes.

the immediates fields with their fields are:
- for unsigned immediates: `u6_imd` (6 bits), `u12_imd` (12 bits).
- for signed immediates: `s9_imd` (9 bits), `s10_imd` (10 bits), `s12_imd` (12 bits), `s19_imd` (19 bits) and `s24_imd` (24 bits).

signed immediates are encoded in twos complement, and all immediates are contiguous inside the instruction except for `s9_imd` where the sign bit is encoded separately.

### shifted register
some instructions take a shifted register oprand, it is encoded in 3 fields: a `gpr` for the register, a `u6_imd` for the shift amount and a `sh` that specifies the shift type.

`sh` is a 2 bit field that can be one of:
- `00`: logical left (`shl`)
- `01`: logical right (`shr`)
- `10`: arithmetic right (`sar`)
- `11`: rotate left (`rol`)

### options
`cw` is a 2 bit field that modifies how a condition is written, it can be:
- **default (`00`)**: overwrite distination register with the computed condition.
- **and (`01`)**: and the computed condition with the distination register condition.
- **or (`11`)**: or the computed condition with the distination register condition.

`ca` is a 1 bit field that modifies how a condition is written, it can be:
- **default (`0`)**: overwrite distination register with the computed condition.
- **and (`1`)**: and the computed condition with the distination register condition.

# arithmatic instructions
## addition
### add (shifted register)
```
add dst:gpr, src1:gpr, src2:sh_reg
```
![add encoding](./ref-assets/add.svg)

adds register `src1` and optionally shifted register `src2` and writes the result to `dst` register.

### add (immediate)
```
add dst:gpr, src1:gpr_pc, src2:u12_imd
```
![add imd encoding](./ref-assets/add_imd.svg)

adds register / `PC` (`src1`) and immediate `src2` and writes the result to `dst` register.

### add.carry
```
add.carry dst:gpr, src1:gpr, src2:gpr, carry:cond
```
![add.carry encoding](./ref-assets/add_carry.svg)

adds registers `src1` and `src2` with carry flag in `carry` register and writes the result to `dst` register, then updates the `carry` register.

### add tripple
```
add dst:gpr, src1:gpr, src2:gpr, src3:gpr
```
![add3 encoding](./ref-assets/add3.svg)

adds registers `src1`, `src2` and `src3` and writes the result to `dst` register.

### cinc
```
cinc dst:gpr, cond:cond, src:gpr
```
![cinc encoding](./ref-assets/cinc.svg)

write to `dst` register the `src` register incremented by `1` if `cond` register is `true`, otherwise move `src` without modification.

## subtract
### sub (shifted register)
```
sub dst:gpr, src1:gpr, src2:sh_reg
```
![sub encoding](./ref-assets/sub.svg)

subtracts optionally shifted register `src2` from register `src1` and writes the result to `dst` register.

### sub reverse (shifted register)
```
sub dst:gpr, src1:sh_reg, src2:gpr
```
![sub rev encoding](./ref-assets/sub_rev.svg)

subtracts register `src2` from optionally shifted register `src1` and writes the result to `dst` register.

### sub (immediate)
```
sub dst:gpr, src1:gpr, src2:u12_imd
```
![sub imd encoding](./ref-assets/sub_imd.svg)

subtracts immediate `src2` from register `src1` and writes the result to `dst` register.

### sub reverse (immediate)
```
sub dst:gpr, src1:u12_imd, src2:gpr
```
![sub rev imd encoding](./ref-assets/sub_rev_imd.svg)

subtracts register `src2` from immediate `src1` and writes the result to `dst` register.

### sub.borrow
```
sub.borrow dst:gpr, src1:gpr, src2:gpr, borrow:cond
```
![sub.borrow encoding](./ref-assets/sub_borrow.svg)

subtracts register `src2` from `src1` with borrow flag in `borrow` register and writes the result to `dst` register, then updates the `borrow` register.

## multiplication
### mult
```
mult dst:gpr, src1:gpr, src2:gpr
```
![mult encoding](./ref-assets/mult.svg)

multiplies registers `src1` by `src2` and writes the result to `dst` register.

it is an alias for `mult.full`

### mult full
```
mult.full plow:gpr, phigh:gpr, src1:gpr, src2:gpr
```
![mult.full encoding](./ref-assets/mult_full.svg)

multiplies registers `src1` by `src2` to produce a full 128 bit product, then writes the low and 64 bits to `plow` and `phigh` registers respectively.

### madd
```
madd dst:gpr, src1:gpr, src2:gpr, src3:gpr
```
![madd encoding](./ref-assets/madd.svg)

multiplies registers `src1` by `src2` and adds register `src3` to the product and writes the result to `dst` register.

### msub
```
msub dst:gpr, src1:gpr, src2:gpr, src3:gpr
```
![msub encoding](./ref-assets/msub.svg)

multiplies registers `src1` by `src2` and subtracts the product from register `src3` and writes the result to `dst` register.

## division
### div
```
div dst:gpr, src1:gpr, src2:gpr
```
![div encoding](./ref-assets/div.svg)

divides registers `src1` by `src2` and writes the quotient to `dst` register.

it is an alias for `div.full`

### rem
```
rem dst:gpr, src1:gpr, src2:gpr
```
![rem encoding](./ref-assets/rem.svg)

computes the remainder of register `src1` divided by register `src2` and writes the result to `dst` register.

it is an alias for `div.full`

### div.full
```
div.full quo:gpr, rem:gpr, src1:gpr, src2:gpr
```
![div.full encoding](./ref-assets/div_full.svg)

divides registers `src1` by `src2` then writes the quotient to `quo` register and the remainder to `rem` register.

### udiv
```
udiv dst:gpr, src1:gpr, src2:gpr
```
![udiv encoding](./ref-assets/udiv.svg)

unsigned divide registers `src1` by `src2` and writes the quotient to `dst` register.

it is an alias for `udiv.full`

### urem
```
urem dst:gpr, src1:gpr, src2:gpr
```
![urem encoding](./ref-assets/urem.svg)

unsigned remainder of register `src1` divided by register `src2` and writes the result to `dst` register.

it is an alias for `udiv.full`

### udiv.full
```
udiv.full quo:gpr, rem:gpr, src1:gpr, src2:gpr
```
![udiv.full encoding](./ref-assets/udiv_full.svg)

unsigned divide registers `src1` by `src2` then writes the quotient to `quo` register and the remainder to `rem` register.

# sign and comparison instructions
## unary operations
### abs
```
abs dst:gpr, src:gpr
```
![abs encoding](./ref-assets/abs.svg)

computes the absolute value of register `src` and writes the result to `dst` register.

### neg
```
neg dst:gpr, src:sh_reg
```
![neg encoding](./ref-assets/neg.svg)

negates an optionally shifted register `src` and writes the result to `dst` register.

it is an alias for `sub`

### cneg
```
cneg dst:gpr, cond:cond, src:gpr
```
![cneg encoding](./ref-assets/cneg.svg)

write to `dst` register the negation of `src` register if `cond` register is `true`, otherwise move `src` without modification.

## min / max
### min
```
min dst:gpr, src1:gpr, src2:gpr
```
![min encoding](./ref-assets/min.svg)

determines the signed minimum of registers `src1` and `src2` and writes it to `dst` register.

### max
```
max dst:gpr, src1:gpr, src2:gpr
```
![max encoding](./ref-assets/max.svg)

determines the signed maximum of registers `src1` and `src2` and writes it to `dst` register.

### umin
```
umin dst:gpr, src1:gpr, src2:gpr
```
![umin encoding](./ref-assets/umin.svg)

determines the unsigned minimum of registers `src1` and `src2` and writes it to `dst` register.

### umax
```
umax dst:gpr, src1:gpr, src2:gpr
```
![umax encoding](./ref-assets/umax.svg)

determines the unsigned maximum of registers `src1` and `src2` and writes it to `dst` register.

## equality comparison
### comp.eq
```
comp.eq{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.eq encoding](./ref-assets/comp_eq.svg)

determines if registers `src1` and `src2` are equal and writes the result condition to `dst` register.

this instruction can optionally and / or the computed condition with `dst`.

### comp.ne
```
comp.ne{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.ne encoding](./ref-assets/comp_ne.svg)

determines if registers `src1` and `src2` are not equal and writes the result condition to `dst` register.

this instruction can optionally and / or the computed condition with `dst`.

### comp.eq immediate
```
comp.eq{_, .and} dst:cond, src1:gpr, src2:s12_imd
```
![comp.eq imd encoding](./ref-assets/comp_eq_imd.svg)

determines if register `src1` is equal to immediate `src2` and writes the result condition to `dst` register.

this instruction can optionally and the computed condition with `dst`.

### comp.ne immediate
```
comp.ne{_, .and} dst:cond, src1:gpr, src2:s12_imd
```
![comp.ne imd encoding](./ref-assets/comp_ne_imd.svg)

determines if register `src1` is not equal to immediate `src2` and writes the result condition to `dst` register.

this instruction can optionally and the computed condition with `dst`.

## signed comparison
### comp.gt
```
comp.gt{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.gt encoding](./ref-assets/comp_gt.svg)

determines if register `src1` is greater than register `src2` and writes the result condition to `dst` register.

this instruction can optionally and / or the computed condition with `dst`.

### comp.le
```
comp.le{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![comp.le encoding](./ref-assets/comp_le.svg)

determines if register `src1` is less than or equal to register `src2` and writes the result condition to `dst` register.

this instruction can optionally and / or the computed condition with `dst`.

### comp.gt immediate
```
comp.gt{_, .and} dst:cond, src1:gpr, src2:s12_imd
```
![comp.gt imd encoding](./ref-assets/comp_gt_imd.svg)

determines if register `src1` is greater than immediate `src2` and writes the result condition to `dst` register.

this instruction can optionally and the computed condition with `dst`.

## unsigned comparison
### ucomp.gt
```
ucomp.gt{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![ucomp.gt encoding](./ref-assets/ucomp_gt.svg)

determines if register `src1` is unsignly greater than register `src2` and writes the result condition to `dst` register.

this instruction can optionally and / or the computed condition with `dst`.

### ucomp.le
```
ucomp.le{_, .and, .or} dst:cond, src1:gpr, src2:gpr
```
![ucomp.le encoding](./ref-assets/ucomp_le.svg)

determines if register `src1` is unsignly less than or equal to register `src2` and writes the result condition to `dst` register.

this instruction can optionally and / or the computed condition with `dst`.

### ucomp.gt immediate
```
ucomp.gt{_, .and} dst:cond, src1:gpr, src2:u12_imd
```
![ucomp.gt imd encoding](./ref-assets/ucomp_gt_imd.svg)

determines if register `src1` is unsignly greater than immediate `src2` and writes the result condition to `dst` register.

this instruction can optionally and the computed condition with `dst`.

# logical instructions

# bit manipulation instructions

# data movement instructions

# load / store instructions

# control flow instructions