//! A small pedagogical compiler for the language Tiny-C
//!
//! This is a Rust translation from Marc Feeley's C version.  I've
//! tried to thread a careful balance between keeping it close to the
//! original while also make it reasonably idomatic clean Rust code.

#![warn(clippy::all, clippy::pedantic)]

// This is a compiler for the Tiny-C language.  Tiny-C is a
// considerably stripped down version of C and it is meant as a
// pedagogical tool for learning about compilers.  The integer global
// variables "a" to "z" are predefined and initialized to zero, and it
// is not possible to declare new variables.  The compiler reads the
// program from standard input and prints out the value of the
// variables that are not zero.  The grammar of Tiny-C in EBNF is:
//
//  <program> ::= <statement>
//  <statement> ::= "if" <paren_expr> <statement> |
//                  "if" <paren_expr> <statement> "else" <statement> |
//                  "while" <paren_expr> <statement> |
//                  "do" <statement> "while" <paren_expr> ";" |
//                  "{" { <statement> } "}" |
//                  <expr> ";" |
//                  ";"
//  <paren_expr> ::= "(" <expr> ")"
//  <expr> ::= <test> | <id> "=" <expr>
//  <test> ::= <sum> | <sum> "<" <sum>
//  <sum> ::= <term> | <sum> "+" <term> | <sum> "-" <term>
//  <term> ::= <id> | <int> | <paren_expr>
//  <id> ::= "a" | "b" | "c" | "d" | ... | "z"
//  <int> ::= <an_unsigned_decimal_integer>
//
//
// Here are a few invocations of the compiler:
//
// % echo "a=b=c=2<3;" | ./a.out
// a = 1
// b = 1
// c = 1
// % echo "{ i=1; while (i<100) i=i+i; }" | ./a.out
// i = 128
// % echo "{ i=125; j=100; while (i-j) if (i<j) j=j-i; else i=i-j; }" | ./a.out
// i = 25
// j = 25
// % echo "{ i=1; do i=i+10; while (i<50); }" | ./a.out
// i = 51
// % echo "{ i=1; while ((i=i+10)<50) ; }" | ./a.out
// i = 51
// % echo "{ i=7; if (i<5) x=1; if (i<10) y=2; }" | ./a.out
// i = 7
// y = 2
//
// The compiler does a minimal amount of error checking to help
// highlight the structure of the compiler.
//

use tinyc_in_rust::{compile_and_run, vm};

fn main() {
    use std::io::BufRead;

    let mut vm = vm::VM::new();

    for line in std::io::stdin().lock().lines() {
        compile_and_run(&mut vm, &line.unwrap());
    }
}
