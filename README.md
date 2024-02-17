# Building an interpreted language in Rust

This project is an implementation of a toy programming language with an interpreter

## Features

- [x] **Lexer**: Tokenizes input to prepare for parsing.
- [x] **Parser**: Analyzes the structure of the code to build an Abstract Syntax Tree (AST).
- [ ] **Evaluator**: Processes the AST to execute the program.
- [x] **REPL**: A Read-Eval-Print Loop for interactive use.

## Getting Started

### Prerequisites

Ensure you have Rust installed on your system. You can download Rust and find installation instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

### Installation

1. Clone the repository:

```sh
git clone https://github.com/stijnVanHorenbeek/return_to_monk.git
```

2. Change into the project directory:

```sh
cd return_to_monk
```

3. Build the project:

```sh
cargo build
```

## Examples

```monkey
// Example of language syntax
let foo = [1, 2, 3, 4, 5];
foo[0] // => 1
let bar = { "name": "stijn", "location": "outer space" };
bar["name"] // => "stijn"

let five = 5;
let ten = 10;
let add = fn(x, y) {
  x + y;
};
let result = add(five, ten);

// recursion
let fibonacci = fn(x) {
   if (x == 0) {
      0
   } else {
      if (x == 1) {
         1
      } else {
         fibonacci(x - 1) + fibonacci(x - 2);
      }
   }
};

// higher order functions
let twice = fn(f, x) {
   return f(f(x));
};
let addTwo = fn(x) {
   return x + 2;
};
twice(addTwo, 2); // => 6
```

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.

## Acknowledgments

- Inspiration from ["Writing An Interpreter In Go" by Thorsten Ball](https://interpreterbook.com). (10/10 book)
- [Top Down Operator Precedence (PRATT parsing)](https://tdop.github.io/)
- [Jonathan Blow & Casey Muratori discussion about operator precedence](https://www.youtube.com/watch?v=fIPO4G42wYE)
