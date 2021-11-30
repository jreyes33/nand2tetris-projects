// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
//
// This program only needs to handle arguments that satisfy
// R0 >= 0, R1 >= 0, and R0*R1 < 32768.

  @R2
  M=0     // R2 = 0

  @R0
  D=M
  @n
  M=D     // n = R0

(LOOP)
  @n
  D=M
  @END
  D;JEQ   // if (n == 0) goto END

  @R1
  D=M
  @R2
  M=M+D   // R2 += R1

  @n
  M=M-1   // n -= 1

  @LOOP
  0;JMP   // goto LOOP

(END)
  @END
  0;JMP   // infinite loop
