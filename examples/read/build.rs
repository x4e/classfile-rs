use std::process::Command;

fn main() {
	Command::new("javac")
		.args(&["TestClass.java"])
		.output()
		.unwrap();
}
