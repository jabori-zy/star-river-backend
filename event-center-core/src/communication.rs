pub mod command;
pub mod response;

use std::hash::Hash;
use strum::IntoEnumIterator;
use std::fmt::Display;

pub use command::Command;
pub use response::Response;



pub trait CommandTarget: Eq + Hash + Clone + IntoEnumIterator + Display {
    fn variants() -> Vec<Self>;
}



pub trait Target {
    type T: CommandTarget;
    fn target(&self) -> &Self::T;
}