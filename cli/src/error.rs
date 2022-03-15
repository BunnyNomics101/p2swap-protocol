//! Module provide application defined errors.

use solana_client::client_error::ClientError;
use solana_sdk::program_error::ProgramError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("RPC client error.")]
    RpcClientError(ClientError),

    #[error("Blockchain program error.")]
    ProgramError(ProgramError),

    #[error("I/O error.")]
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<ClientError> for Error {
    fn from(error: ClientError) -> Self {
        Error::RpcClientError(error)
    }
}

impl From<ProgramError> for Error {
    fn from(error: ProgramError) -> Self {
        Error::ProgramError(error)
    }
}
