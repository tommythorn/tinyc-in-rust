# Tiny-C compiler, in Rust

As an exercize in Rust, rewrote Marc Feeley's [Tiny-C
compile](https://www.iro.umontreal.ca/~felipe/IFT2030-Automne2002/Complements/tinyc.c)
(Copyright (C) 2001 by Marc Feeley, All Rights Reserved).  The Rust
translation has been kept as close in spirit as possible to the
original, while still being reasonable example of Rust (please do let
me know what could be improved).

Reproducing the language description here:

This is a compiler for the Tiny-C language.  Tiny-C is a considerably
stripped down version of C and it is meant as a pedagogical tool for
learning about compilers.  The integer global variables "a" to "z" are
predefined and initialized to zero, and it is not possible to declare
new variables.  The compiler reads the program from standard input and
prints out the value of the variables that are not zero.  The grammar
of Tiny-C in EBNF is:

``` BNF
 <program> ::= <statement>
 <statement> ::= "if" <paren_expr> <statement> |
                 "if" <paren_expr> <statement> "else" <statement> |
                 "while" <paren_expr> <statement> |
                 "do" <statement> "while" <paren_expr> ";" |
                 "{" { <statement> } "}" |
                 <expr> ";" |
                 ";"
 <paren_expr> ::= "(" <expr> ")"
 <expr> ::= <test> | <id> "=" <expr>
 <test> ::= <sum> | <sum> "<" <sum>
 <sum> ::= <term> | <sum> "+" <term> | <sum> "-" <term>
 <term> ::= <id> | <int> | <paren_expr>
 <id> ::= "a" | "b" | "c" | "d" | ... | "z"
 <int> ::= <an_unsigned_decimal_integer>
```

Here are a few invocations of the compiler:

``` SH
$ echo "a=b=c=2<3;" | cargo run
a = 1
b = 1
c = 1
$ echo "{ i=1; while (i<100) i=i+i; }" | cargo run
i = 128
$ echo "{ i=125; j=100; while (i-j) if (i<j) j=j-i; else i=i-j; }" | cargo run
i = 25
j = 25
$ echo "{ i=1; do i=i+10; while (i<50); }" | cargo run
i = 51
$ echo "{ i=1; while ((i=i+10)<50) ; }" | cargo run
i = 51
$ echo "{ i=7; if (i<5) x=1; if (i<10) y=2; }" | cargo run
i = 7
y = 2
```

The compiler does a minimal amount of error checking to help highlight
the structure of the compiler.
