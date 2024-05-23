use crate::shared::settings::EntraSettings;
use log::debug;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AdditionalClaims, ClientId, ClientSecret, IssuerUrl, ProviderMetadata, RedirectUrl,
};
use serde::{Deserialize, Serialize};

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
