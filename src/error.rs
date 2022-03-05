use crate::parameter::Parameter;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to write item {parameter:?}"))]
    Write { parameter: Parameter },
}
