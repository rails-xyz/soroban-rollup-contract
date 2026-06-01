#![no_std]

mod contract;
#[cfg(feature = "certora")]
mod certora;
#[cfg(test)]
mod test;
