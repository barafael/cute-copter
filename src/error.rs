use cute_copter_config_proto::parameter::Write;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to write item {parameter:?}"))]
    Write { parameter: Write },
}
