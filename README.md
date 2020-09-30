# legv8debug
A debugger for legv8 assembly.

### Supported commands
- `q`: quit the debugger
- `r`: run the program until a breakpoint is hit or the program terminates
- `b X`: set a breakpoint at line X.
- `s [X]`: run X instructions. If no X is provided defaults to 1.
- `d`: prints registers and memory values in a format similar to hexdump, little-endian
- `p X`: prints the contents of register X in little-endian hex and decimal.

### Currently implemented instructions
- PRNT, PRN, DUMP - treated as nops.
- ADDI
- ADD
- SUB
- SUBI
- SUBS
- CBZ
- CBNZ
- B
- B.EQ
- B.GT
- B.GE
- B.LT
- B.LE
- BL
- BR
- STUR
- LDUR
- LSL
- LSR
