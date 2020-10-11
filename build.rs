use std::process::Command;

fn main() {
	Command::new("javac")
		.args(&["Test.java"])
		.output()
		.expect("failed to execute process");
}
