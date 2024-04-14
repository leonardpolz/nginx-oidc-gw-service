use getset::Getters;
use openidconnect::{Nonce, PkceCodeVerifier};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct OidcState {
    redirect_uri: String,
    pkce_verifier: PkceCodeVerifier,
    nonce: Nonce,
}

impl OidcState {
    pub fn new(redirect_uri: String, pkce_verifier: PkceCodeVerifier, nonce: Nonce) -> OidcState {
        OidcState {
            redirect_uri,
            pkce_verifier: pkce_verifier,
            nonce: nonce,
        }
    }

    pub fn take_pkce_verifier(self) -> PkceCodeVerifier {
        self.pkce_verifier
    }

    pub fn take_nonce(self) -> Nonce {
        self.nonce
    }
}
