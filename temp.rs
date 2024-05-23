use anyhow::{anyhow, Result};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    core::{CoreClient, CoreGenderClaim, CoreIdToken, CoreProviderMetadata, CoreResponseType},
    AuthorizationCode, ClientId, ClientSecret, IdTokenClaims, IssuerUrl, Nonce,
    OAuth2TokenResponse, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use tokio::main;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MyAdditionalClaims {
    pub oid: String,
}

impl AdditionalClaims for MyAdditionalClaims {}

pub async fn init_oidc_client(entra_settings: &EntraSettings) -> CoreClient {
    debug!("Initializing OIDC client...");

    let issuer_url = IssuerUrl::new(format!(
        "https://login.microsoftonline.com/{}/v2.0",
        entra_settings.tenant_id().to_string()
    ))
    .unwrap();

    // Discover the provider metadata from the issuer URL
    debug!("Discovering provider metadata from issuer URL...");
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .unwrap();

    // Create a new OIDC client from the provider metadata
    debug!("Creating OIDC client from provider metadata...");
    CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(entra_settings.client_id().to_string()),
        Some(ClientSecret::new(
            entra_settings.client_secret().to_string(),
        )),
    )
    .set_redirect_uri(RedirectUrl::new(entra_settings.redirect_uri().to_string()).unwrap())
}

async fn handle_authentication(
    entra_settings: &EntraSettings,
    oidc_code: String,
    oidc_state: OidcState,
    db_context: &DbContext,
) -> Result<()> {
    let oidc_client = init_oidc_client(entra_settings).await;

    // Exchange the authorization code for tokens
    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(oidc_code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .expect("Failed to exchange code for token");

    // Extract the ID token
    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))
        .expect("Failed to get ID token");

    // Extract the nonce
    let nonce = oidc_state.take_nonce().expect("Nonce not found");

    // Parse the ID token claims with MyAdditionalClaims
    let claims: IdTokenClaims<MyAdditionalClaims, CoreGenderClaim> = id_token
        .claims(&oidc_client.id_token_verifier(), &nonce)
        .expect("Failed to verify ID token");

    // Print user authentication info
    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map(|email| email.as_str())
            .unwrap_or("<not provided>"),
    );

    // Access the `oid` claim from additional claims
    let additional_claims = claims.additional_claims();
    let oid = &additional_claims.oid;

    println!("OID: {}", oid);

    // Fetch the user by `oid` from the database (assume `db_context` is properly initialized)
    let user_result = db_context
        .fetch_user_by_oid(oid.clone())
        .await
        .expect("Failed to fetch user");

    println!("User fetched from database: {:?}", user_result);

    Ok(())
}
