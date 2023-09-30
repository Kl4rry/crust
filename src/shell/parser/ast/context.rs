use std::sync::Arc;

use crate::{
    parser::source::Source,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub struct Context<'a> {
    pub shell: &'a mut Shell,
    pub frame: Frame,
    pub output: &'a mut OutputStream,
    pub input: &'a mut ValueStream,
    pub src: Arc<Source>,
}
