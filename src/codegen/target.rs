use std::fmt::Write as _;

#[derive(Default)]
pub struct CodegenTarget {
    indentation: String,
    output: String,
}

impl CodegenTarget {
    pub fn write_line(&mut self, v: &str) {
        self.output.write_str(&self.indentation).expect("Codegen");
        self.output.write_str(v).expect("Codegen");
        self.output.write_char('\n').expect("Codegen");
    }

    pub fn increase_indent(&mut self) {
        self.indentation.push_str("  ");
    }
    pub fn decrease_indent(&mut self) {
        self.indentation.pop();
        self.indentation.pop();
    }

    pub fn into_string(self) -> String {
        self.output
    }
}
