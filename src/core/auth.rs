use crate::error::Error;

pub struct Claim {
    user: String,
}

pub trait Author {
    fn hash_password(&self, password: &str, salt: &str) -> Result<String, Error>;
    fn gen_token(&self, claim: &Claim) -> Result<String, Error>;
    fn verify_token(&self, token: &str) -> Result<Claim, Error>;
}
