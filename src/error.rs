use cute_copter_config_proto::parameter::ConfigurationCommand;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to write item {parameter:?}"))]
    Write { parameter: ConfigurationCommand },
}
