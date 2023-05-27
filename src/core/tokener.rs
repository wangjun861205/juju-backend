use crate::error::Error;
use serde::{Deserialize, Serialize};

pub trait Payload: Serialize + for<'d> Deserialize<'d> {
    fn user(&self) -> &str;
}

pub trait Tokener<P: Payload> {
    fn gen_token(&self, payload: &P) -> Result<String, Error>;
    fn verify_token(&self, token: &str) -> Result<P, Error>;
}
