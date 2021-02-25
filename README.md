# classfile-rs
Speedy class file implementation for rust

This is a library for reading, manipulating, and writing class files.

Goals:
* Simple, abstract AST
* Stackmap frame generation
* Resistant to malicious/malformed class files

The AST is designed to be more logical and intuitive to a Java developer than the official bytecode
 specification. 

For example, `iconst_0`s are represented as `ldc(0)`s, `invokestatic(...)`s are represented as `invoke(static, ...)`.
Bytecode offsets are also transformed into labels.
The constant pool is fully abstracted.

Todos:
* Stackmap frame generation
* Class writing
(Feel free to contribute on these)

## Speed
The library should take <1ms to read an averagely sized class file, but can take longer depending on the amount of
 control flow within the class (Label reading is not optimal at the moment).

Here is a benchmark:
![Throughput benchmark](https://cdn.discordapp.com/attachments/665688984302649354/803225667399057448/unknown.png)

## Examples
[Reading a class file](https://github.com/x4e/classfile-rs/tree/master/examples/read/src/main.rs)

