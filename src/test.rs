#[cfg(test)]
mod tests {
    use std::fs;

    use crate::shell::{
        stream::{OutputStream, ValueStream},
        Shell,
    };
    #[test]
    fn basic_test() {
        let mut shell = Shell::new(Vec::new());
        shell.run_src(
            "it works".into(),
            "assert (1 == 1)".into(),
            &mut OutputStream::new_capture(),
            ValueStream::new(),
        );
        assert!(shell.status() == 0);
    }

    #[test]
    fn language_test() {
        for _ in 0..10 {
            for entry in glob::glob("tests/*.crust").unwrap() {
                let path = entry.unwrap();
                let file = fs::read_to_string(&path).unwrap();
                let mut shell = Shell::new(Vec::new());
                shell.run_src(
                    path.to_str().unwrap().into(),
                    file,
                    &mut OutputStream::new_capture(),
                    ValueStream::new(),
                );
                assert_eq!(0, shell.status());
            }
        }
    }
}
