use subprocess::ExitStatus;

pub trait ExitStatusExt {
    fn code(&self) -> i64;
    fn to_string(&self) -> String;
}

impl ExitStatusExt for ExitStatus {
    fn code(&self) -> i64 {
        match self {
            ExitStatus::Exited(code) => *code as i64,
            ExitStatus::Signaled(code) => *code as i64,
            ExitStatus::Other(code) => *code as i64,
            ExitStatus::Undetermined => 1,
        }
    }

    fn to_string(&self) -> String {
        match self {
            ExitStatus::Exited(code) => format!("{code}"),
            ExitStatus::Signaled(code) => format!("{code}"),
            ExitStatus::Other(code) => format!("{code}"),
            ExitStatus::Undetermined => "1".into(),
        }
    }
}
