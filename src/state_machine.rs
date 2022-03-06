use crate::error::Error;
use core::marker::PhantomData;
use cute_copter_config_proto::{configuration::Config, parameter::Write};
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use stm32f1xx_hal::flash::FlashWriter;

pub fn load_from_flash(flash: &mut FlashWriter) -> Option<Config> {
    let bytes = flash
        .read(127 * 1024, core::mem::size_of::<Config>())
        .unwrap();
    Some(from_bytes(bytes).unwrap())
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Disarmed;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Armed;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Copter<State> {
    config: Config,
    marker: PhantomData<State>,
}

impl Copter<Disarmed> {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            marker: PhantomData::default(),
        }
    }

    pub fn write(&mut self, write: Write) -> Result<(), Error> {
        match write {
            Write::RollProportional(value) => todo!(),
            _ => todo!(),
        }
    }

    pub fn read(&self, write: Write) -> Result<Write, Error> {
        todo!()
    }

    pub fn arm(self, writer: &mut FlashWriter) -> Result<Copter<Armed>, (Self, Error)> {
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
