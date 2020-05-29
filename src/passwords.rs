extern crate base64;
extern crate ring;
use ring::rand::{SecureRandom, SystemRandom};
use ring::{digest, pbkdf2};
use std::error::Error;
use std::fmt;

lazy_static! {
    static ref RANDOMNESS_SOURCE: SystemRandom = {
        let mut random_bytes = [0; 1];
        let randomness_source = SystemRandom::new();
        randomness_source.fill(&mut random_bytes).unwrap(); // Initialize source of randomness

        randomness_source
    };
}
static DIGEST_ALG: &'static digest::Algorithm = &digest::SHA512;
const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
const ITERATIONS: u32 = 100_000;

pub struct PasswordHash {
    hash: Vec<u8>,
    iterations: u32,
    salt: Vec<u8>,
}

impl PasswordHash {
    fn new(hash: Vec<u8>, iterations: u32, salt: Vec<u8>) -> PasswordHash {
        PasswordHash {
            hash,
            iterations,
            salt,
        }
    }

    pub fn from(string: &str) -> Result<PasswordHash, PasswordError> {
        let mut hash_parts = string.split("$");
        let encoded_salt = hash_parts
            .next()
            .ok_or_else(|| PasswordError::new("The hash does not contain the salt"))?;
        let salt = match base64::decode(encoded_salt) {
            Ok(salt) => salt,
            Err(_) => return Err(PasswordError::new("The salt contains invalid characters")),
        };

        let iterations = match hash_parts
            .next()
            .ok_or_else(|| PasswordError::new("The hash does not contain the iteration count"))?
            .to_string()
            .parse::<u32>()
        {
            Ok(iterations) => iterations,
            Err(_) => {
                return Err(PasswordError::new(
                    "The iteration count was not a valid u32",
                ))
            }
        };

        let encoded_password_hash = hash_parts
            .next()
            .ok_or_else(|| PasswordError::new("The hash does not contain the hashed password"))?;
        let password_hash = match base64::decode(encoded_password_hash) {
            Ok(password_hash) => password_hash,
            Err(_) => {
                return Err(PasswordError::new(
                    "The hashed password contains invalid characters",
                ))
            }
        };

        Ok(PasswordHash {
            hash: password_hash,
            iterations,
            salt,
        })
    }

    pub fn to_string(&self) -> String {
        [
            base64::encode(&self.salt),
            ITERATIONS.to_string(),
            base64::encode(&self.hash),
        ]
        .join("$")
    }
}

/// Returns a secure hash of the input password containing the salt, iterations and hashed password.
///
/// The returned hash is of the form "base64(salt)$iterations$base64(hash(password.as_bytes()))",
/// where the salt consists of 16 random bytes (generated by the operating system), iterations is
/// the iteration count of the pbkdf and password is the user-supplied password.
pub fn hash_password(password: &str) -> Result<PasswordHash, PasswordError> {
    let mut salt: [u8; 16] = [0; 16];
    match RANDOMNESS_SOURCE.fill(&mut salt) {
        Ok(_) => (),
        Err(_) => {
            return Err(PasswordError::new(
                "Could not get random bytes to generate a salt",
            ))
        }
    };

    let mut hashed_bytes: [u8; CREDENTIAL_LEN] = [0; CREDENTIAL_LEN];

    pbkdf2::derive(
        DIGEST_ALG,
        ITERATIONS,
        &salt,
        password.as_bytes(),
        &mut hashed_bytes,
    );

    Ok(PasswordHash::new(
        hashed_bytes.to_vec(),
        ITERATIONS,
        salt.to_vec(),
    ))
}

/// Returns a result containing the unit type if the provided password matches the hash, or an error.
///
/// For the unit type to be returned, the provided hash must contain all necessary data to compute
/// whether or not there is a match, such as the salt, iteration count and hashed password itself.
/// See `hash_password()` for the format of this hash.
pub fn verify_password(password: &str, hash: &PasswordHash) -> Result<(), PasswordError> {
    match pbkdf2::verify(
        DIGEST_ALG,
        hash.iterations,
        &hash.salt,
        password.as_bytes(),
        &hash.hash,
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(PasswordError::new("Password is incorrect")),
    }
}

#[derive(Debug)]
pub struct PasswordError {
    details: String,
}

impl PasswordError {
    fn new(msg: &str) -> PasswordError {
        PasswordError {
            details: String::from(msg),
        }
    }
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.details)
    }
}

impl Error for PasswordError {}