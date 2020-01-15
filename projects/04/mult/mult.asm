// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)

// Put your code here.

@R2
M=0 // initialise result to 0

@sum
M=0 // sum = 0

@i
M=0 // i = 0

(LOOP)
    // if i = R1, go to END
    @i
    D=M
    @R1
    D=D-M
    @PROD
    D;JEQ

    // add R0 to sum
    @sum
    D=M
    @R0
    D=D+M
    @sum
    M=D

    // increment i
    @i
    M=M+1

    // keep looping
    @LOOP
    0;JMP

(PROD)
    // set R2 to product so far
    @sum
    D=M
    @R2
    M=D
    @END
    0;JMP

(END)
    // loop forever
    @END
    0;JMP