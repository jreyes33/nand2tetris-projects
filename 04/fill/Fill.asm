// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed.
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

(LOOP)
  @KBD
  D=M
  @WHITE
  D;JEQ   // if (KBD == 0) goto WHITE

  @color  // otherwise, paint it black
  M=-1    // color = -1

  @PAINT
  0;JMP   // goto PAINT

(WHITE)
  @color
  M=0     // color = 0

(PAINT)
  @8191
  D=A
  @i
  M=D     // i = 8191

(KEEP_PAINTING)
  @i
  D=M
  @SCREEN
  D=A+D
  @p      // p is for pointer… or painter
  M=D     // p = SCREEN + i

  @color
  D=M
  @p
  A=M
  M=D     // RAM[p] = color

  @i
  M=M-1   // i -= 1

  D=M
  @KEEP_PAINTING
  D;JGE   // if (i >= 0) goto KEEP_PAINTING

  @LOOP
  0;JMP   // goto LOOP
