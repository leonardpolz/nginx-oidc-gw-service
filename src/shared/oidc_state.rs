use getset::Getters;
use openidconnect::{Nonce, PkceCodeVerifier};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct OidcState {
    redirect_uri: String,
    pkce_verifier: Option<PkceCodeVerifier>,
    nonce: Option<Nonce>,
}

impl OidcState {
    pub fn new(redirect_uri: String, pkce_verifier: PkceCodeVerifier, nonce: Nonce) -> OidcState {
        OidcState {
            redirect_uri,
            pkce_verifier: Some(pkce_verifier),
            nonce: Some(nonce),
        }
    }

    pub fn take_pkce_verifier(&mut self) -> Option<PkceCodeVerifier> {
        self.pkce_verifier.take()
    }

    pub fn take_nonce(&mut self) -> Option<Nonce> {
        self.nonce.take()
    }
}
