#![no_std]

#[cfg(feature = "certora")]
mod certora;
mod contract;
#[cfg(test)]
mod test;
