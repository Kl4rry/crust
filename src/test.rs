#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc};

    use miette::NamedSource;

    use crate::{
        parser::{lexer::Lexer, Parser},
        shell::{
            stream::{OutputStream, ValueStream},
            Shell,
        },
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

    fn random_ascii_string(len: usize) -> String {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        let mut s = String::new();
        for _ in 0..len {
            let ch: u8 = rng.gen_range(0..=127);
            s.push(ch as char);
        }
        s
    }

    #[test]
    fn random_ascii_lex_test() {
        for _ in 0..100 {
            let string = random_ascii_string(1000);
            eprintln!("{string}");
            let lexer = Lexer::new(Arc::new(NamedSource::new("random ascii", string)));
            let tokens: Vec<_> = lexer.collect();
            assert!(!tokens.is_empty());
        }
    }

    #[test]
    fn random_ascii_parse_test() {
        for _ in 0..100 {
            let string = random_ascii_string(1000);
            eprintln!("{string}");
            let parser = Parser::new("random ascii".into(), string);
            let _ = parser.parse();
        }
    }

    #[test]
    fn control_char_fail_parse() {
        let mut string = String::new();
        for ch in 0u8..0x1F {
            string.push(ch as char);
        }
        let parser = Parser::new("control chars".into(), string);
        assert!(!parser.parse().1.is_empty());
    }
}
