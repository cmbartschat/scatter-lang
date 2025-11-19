use std::fmt::Write;

#[derive(Default)]
pub struct CodegenTarget {
    indentation: String,
    output: String,
}

impl CodegenTarget {
    pub fn write_line(&mut self, v: &str) {
        self.output.write_str(&self.indentation).unwrap();
        self.output.write_str(v).unwrap();
        self.output.write_char('\n').unwrap();
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
