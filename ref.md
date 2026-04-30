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
