# classfile-rs
Speedy class file implementation for rust

This is a library for reading, manipulating, and writing class files.

Goals:
* Simple, abstract AST
* Stackmap frame generation
* Resistant to malicious/malformed class files

The AST is designed to be more logical and intuitive than the official bytecode specification. 
For example, `iconst_0`s are represented as `ldc(0)`s, `incokestatic(...)`s are represented as `invoke(static, ...)`.
Bytecode offsets are also transformed into labels.
The constant pool is fully abstracted.

Todos:
* Stackmap frame generation
* Class writing
(Feel free to contribute on these)
