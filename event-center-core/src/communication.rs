pub mod command;
pub mod response;

use std::{fmt::Display, hash::Hash};

pub use command::Command;
pub use response::Response;
use strum::IntoEnumIterator;

pub trait CommandTarget: Eq + Hash + Clone + IntoEnumIterator + Display {
    fn variants() -> Vec<Self>;
}

pub trait Target {
    type T: CommandTarget;
    fn target(&self) -> &Self::T;
}
