use crate::shell::{frame::Frame, stream::OutputStream, Shell};

pub struct Context<'a> {
    pub shell: &'a mut Shell,
    pub frame: Frame,
    pub output: &'a mut OutputStream,
}
