#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature="rust-evm")]
mod rust_evm;
#[cfg(feature="sputnik-evm")]
mod sputnik_evm;
mod state;
mod types;

pub use rosetta_ethereum_backend as backend;
pub use rosetta_ethereum_primitives as primitives;
pub use state::{StateDB, PrefetchError};
pub use types::{ExecutionResult, ExitError, ExitSucceed, Log, ExecutionError, ExecutionReverted, ExecutionSucceed};

#[cfg(any(feature="rust-evm",feature="sputnik-evm"))]
pub mod vms {
    #[cfg(feature="rust-evm")]
    pub use super::rust_evm::{Executor as RustEVM, Error, EvmError};
    
    #[cfg(feature="sputnik-evm")]
    pub use super::sputnik_evm::{SputnikExecutor as SputnikEVM, SputnikConfig};
}

pub trait Executor {
    type Error: alloc::fmt::Display;

    #[allow(clippy::missing_errors_doc)]
    fn execute(
        &mut self,
        tx: &backend::TransactionCall,
        at: backend::AtBlock,
    ) -> Result<backend::ExitReason, Self::Error>;
}

#[cfg(test)]
mod tests {
    
}
