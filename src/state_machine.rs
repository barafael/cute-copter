use crate::{error::Error, parameter::Parameter};
use core::marker::PhantomData;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Disarmed;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Armed;

pub struct Copter<State> {
    marker: PhantomData<State>,
}

impl Copter<Disarmed> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData::default(),
        }
    }

    pub fn write(&mut self, parameter: Parameter) -> Result<(), Error> {
        match parameter {
            Parameter::RollProportional(value) => todo!(),
        }
    }

    pub fn read(&self, parameter: Parameter) -> Result<Parameter, Error> {
        todo!()
    }

    pub fn arm(self, storage: &mut Storage) -> Result<Copter<Armed>, (Self, Error)> {
        // TODO persist data to storage
        Ok(Copter {
            marker: PhantomData::<Armed>,
        })
    }
}

impl Copter<Armed> {
    pub fn disarm(self) -> Result<Copter<Disarmed>, Self> {
        Ok(Copter {
            marker: PhantomData::<Disarmed>,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Storage;
