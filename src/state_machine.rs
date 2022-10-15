use core::marker::PhantomData;
use cute_copter_config_proto::{command::SetParameter, configuration::Configuration, error::Error};
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use stm32f1xx_hal::flash::FlashWriter;

pub fn load_from_flash(flash: &mut FlashWriter) -> Option<Configuration> {
    let bytes = flash
        .read(127 * 1024, core::mem::size_of::<Configuration>())
        .unwrap();
    Some(from_bytes(bytes).unwrap())
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Disarmed;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Armed;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Copter<State> {
    config: Configuration,
    marker: PhantomData<State>,
}

impl Copter<Disarmed> {
    pub fn from_config(config: Configuration) -> Self {
        Self {
            config,
            marker: PhantomData::default(),
        }
    }

    pub fn set_parameter(&mut self, param: SetParameter) -> Result<(), Error> {
        match param {
            SetParameter::RollProportional(value) => todo!(),
            _ => todo!(),
        }
    }

    pub fn read(&self, write: SetParameter) -> Result<SetParameter, Error> {
        todo!()
    }

    pub fn arm(self, writer: &mut FlashWriter) -> Result<Copter<Armed>, (Self, Error)> {
        // Maybe persist data only when disarming.
        // Actually, may be better to have an explicit command for persisting the config.
        let output: Vec<u8, 36> = to_vec(&self.config).unwrap();

        writer.erase(127 * 1024, 256).unwrap();
        writer.write(127 * 1024, &output).unwrap();
        Ok(Copter {
            config: self.config,
            marker: PhantomData::<Armed>,
        })
    }
}

impl Copter<Armed> {
    pub fn disarm(self) -> Result<Copter<Disarmed>, Self> {
        Ok(Copter {
            config: self.config,
            marker: PhantomData::<Disarmed>,
        })
    }
}
