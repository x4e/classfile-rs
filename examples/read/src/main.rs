use classfile::classfile::ClassFile;
use classfile::error::Result;

use std::fs::{File};
use std::io::{BufReader};


/// This example will read a class file from disc and print it
fn main() -> Result<()> {
	let f = File::open("TestClass.class").unwrap();
	let mut reader = BufReader::new(f);
	let class = ClassFile::parse(&mut reader)?;
	println!("{:#x?}", class);
	
	Ok(())
}
/// Output:
/// ClassFile {
//     magic: 0xcafebabe,
//     version: ClassVersion {
//         major: JAVA_15,
//         minor: 0x0,
//     },
//     access_flags: PUBLIC,
//     this_class: "TestClass",
//     super_class: Some(
//         "java/lang/Object",
//     ),
//     interfaces: [],
//     fields: [],
//     methods: [
//         Method {
//             access_flags: PUBLIC,
//             name: "<init>",
//             descriptor: "()V",
//             signature: None,
//             exceptions: [],
//             code: Some(
//                 CodeAttribute {
//                     max_stack: 0x1,
//                     max_locals: 0x1,
//                     insns: [
//                         LocalLoadInsn {
//                             kind: Reference,
//                             index: 0x0,
//                         },
//                         InvokeInsn {
//                             kind: Special,
//                             class: "java/lang/Object",
//                             name: "<init>",
//                             descriptor: "()V",
//                             interface_method: false,
//                         },
//                         ReturnInsn {
//                             kind: Void,
//                         },
//                     ],
//                     exceptions: [],
//                     attributes: [
//                         Unknown(
//                             UnknownAttribute {
//                                 name: "LineNumberTable",
//                                 buf: [
//                                     0x0,
//                                     0x1,
//                                     0x0,
//                                     0x0,
//                                     0x0,
//                                     0x1,
//                                 ],
//                             },
//                         ),
//                     ],
//                 },
//             ),
//             attributes: [],
//         },
//         Method {
//             access_flags: PUBLIC | STATIC,
//             name: "main",
//             descriptor: "([Ljava/lang/String;)V",
//             signature: None,
//             exceptions: [],
//             code: Some(
//                 CodeAttribute {
//                     max_stack: 0x2,
//                     max_locals: 0x1,
//                     insns: [
//                         GetFieldInsn {
//                             instance: false,
//                             class: "java/lang/System",
//                             name: "out",
//                             descriptor: "Ljava/io/PrintStream;",
//                         },
//                         LdcInsn {
//                             constant: String(
//                                 "Hello, World!",
//                             ),
//                         },
//                         InvokeInsn {
//                             kind: Instance,
//                             class: "java/io/PrintStream",
//                             name: "println",
//                             descriptor: "(Ljava/lang/String;)V",
//                             interface_method: false,
//                         },
//                         ReturnInsn {
//                             kind: Void,
//                         },
//                     ],
//                     exceptions: [],
//                     attributes: [
//                         Unknown(
//                             UnknownAttribute {
//                                 name: "LineNumberTable",
//                                 buf: [
//                                     0x0,
//                                     0x2,
//                                     0x0,
//                                     0x0,
//                                     0x0,
//                                     0x3,
//                                     0x0,
//                                     0x8,
//                                     0x0,
//                                     0x4,
//                                 ],
//                             },
//                         ),
//                     ],
//                 },
//             ),
//             attributes: [],
//         },
//     ],
//     attributes: [
//         SourceFile(
//             SourceFileAttribute {
//                 source_file: "TestClass.java",
//             },
//         ),
//     ],
// }
