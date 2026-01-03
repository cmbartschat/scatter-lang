#[cfg(test)]
mod tests {

    #[cfg(test)]
    use std::path::PathBuf;

    use assert_cmd::cargo::cargo_bin_cmd;
    use rstest::*;

    #[derive(Debug)]
    pub struct IntegrationTest<'a> {
        command: &'a str,
        stdin: &'a str,
        stdout: &'a str,
        stderr: &'a str,
        exit_code: i32,
    }

    impl<'a> IntegrationTest<'a> {
        pub fn parse(source: &'a str) -> Result<IntegrationTest<'a>, String> {
            let advance_to = |s: &'a str, expected: &str| -> Result<(&'a str, &'a str), String> {
                let pattern = format!("\n{expected}\n");
                let Some(index) = s.find(&pattern) else {
                    return Err(format!("Could not find: {expected}"));
                };

                let start = usize::from(index > 0 && s.starts_with('\n'));
                Ok((&s[start..index], &s[(index + pattern.len() - 1)..]))
            };

            let (command, remainder) = advance_to(source, "STDIN")?;
            let (stdin, remainder) = advance_to(remainder, "STDOUT")?;
            let (stdout, remainder) = advance_to(remainder, "STDERR")?;
            let (stderr, remainder) = advance_to(remainder, "EXIT_CODE")?;
            let exit_code: i32 = remainder.trim().parse().unwrap_or_default();

            Ok(Self {
                command,
                stdin,
                stdout,
                stderr,
                exit_code,
            })
        }

        pub fn test(&self) {
            let mut cmd = cargo_bin_cmd!();
            cmd.args(self.command.lines());
            cmd.write_stdin(self.stdin);
            let output = cmd.output().unwrap();
            let (stdout, stderr, exit_code) = (output.stdout, output.stderr, output.status);

            if self.stderr != "*" {
                assert_eq!(
                    String::from_utf8(stderr).unwrap(),
                    self.stderr,
                    "Incorrect Stderr"
                );
            }
            if self.stdout != "*" {
                assert_eq!(
                    String::from_utf8(stdout).unwrap(),
                    self.stdout,
                    "Incorrect Stdout"
                );
            }

            assert_eq!(
                exit_code.code().unwrap(),
                self.exit_code,
                "Incorrect exit code"
            );
        }
    }

    fn check_integration(definition: &str) {
        let integration = IntegrationTest::parse(definition).unwrap();
        integration.test();
    }

    #[rstest]
    fn integration(#[files("int/*.txt")] path: PathBuf) {
        check_integration(std::fs::read_to_string(path).unwrap().as_str());
    }
}
