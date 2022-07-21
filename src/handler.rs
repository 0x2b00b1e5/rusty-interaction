use crate::security::*;

#[cfg(feature = "extended-handler")]
use crate::types::application::*;

#[cfg(feature = "handler")]
use crate::types::interaction::*;

#[cfg(feature = "extended-handler")]
use crate::types::HttpError;
use crate::types::Snowflake;
#[cfg(feature = "extended-handler")]
use crate::{
    expect_specific_api_response, expect_successful_api_response,
    expect_successful_api_response_and_return,
};
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use reqwest::header;
use reqwest::Client;

use log::{debug, error};
use std::fmt;

use anymap::{any::CloneAny, Map};

use ed25519_dalek::PublicKey;

use rustls::ServerConfig;

use std::{collections::HashMap, future::Future, pin::Pin, sync::Mutex};

type AnyMap = Map<dyn CloneAny + Send + Sync>;

/// Alias for InteractionResponse
pub type HandlerResponse = Result<InteractionResponse, std::convert::Infallible>;

type HandlerFunction = fn(
    &mut InteractionHandler,
    Context,
) -> Pin<Box<dyn Future<Output = HandlerResponse> + Send + '_>>;

macro_rules! match_handler_response {
    ($response:ident) => {

        if let Ok(ref __unwrapped_response__) = $response{
            match __unwrapped_response__.r#type {
                InteractionResponseType::None => {
                    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
                }
                InteractionResponseType::DefferedChannelMessageWithSource
                | InteractionResponseType::DefferedUpdateMessage => {
                    /* The use of HTTP code 202 is more appropriate when an Interaction is deffered.
                    If an application is first sending a deffered channel message response, this usually means the system
                    is still processing whatever it is doing.
                    See the spec: https://tools.ietf.org/html/rfc7231#section-6.3.3 */
                    Ok(HttpResponse::build(StatusCode::ACCEPTED).json(__unwrapped_response__))
                }
                _ => {
                    // Send out a response to Discord
                    let r = HttpResponse::build(StatusCode::OK).json(__unwrapped_response__);

                    Ok(r)
                }
            }
        }
        else{
            debug!("Responding with 500!");
            Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish())
        }

    };
}

#[cfg(feature = "handler")]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Used for some functions to define which scope should be manipulated.
pub enum ManipulationScope {
    /// Only apply changes locally
    Local,
    /// Apply changes locally and to Discord
    All,
    /// Only apply changes with Discord
    Discord,
}

#[cfg(feature = "handler")]
#[derive(Clone)]
/// The InteractionHandler is the 'thing' that will handle your incoming interactions.
/// It does interaction validation (as required by Discord) and provides a pre-defined actix-web server
/// with [`InteractionHandler::run`] and [`InteractionHandler::run_ssl`]
pub struct InteractionHandler {
    application_id: Snowflake,

    app_public_key: PublicKey,
    client: Client,

    global_handles: HashMap<&'static str, HandlerFunction>,
    component_handles: HashMap<&'static str, HandlerFunction>,

    // These handles are 'forgotten' every time the app is shutdown (whatever the reason may be).
    guild_handles: HashMap<Snowflake, HandlerFunction>,

    /// Field to access data
    pub data: AnyMap,
}

#[cfg(feature = "handler")]
// Only here to make Debug less generic, so I can send a reference
impl fmt::Debug for InteractionHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f
            .debug_struct("InteractionHandler")
            .field("app_public_key", &self.app_public_key)
            .field("global_handles_len", &self.global_handles.len())
            .field("component_handles_len", &self.component_handles.len())
            .finish();
    }
}

#[cfg(feature = "handler")]
impl InteractionHandler {
    /// Initalizes a new `InteractionHandler`
    pub fn new(
        app_id: Snowflake,
        pbk_str: impl AsRef<str>,
        token: Option<&String>,
    ) -> InteractionHandler {
        let pbk_bytes =
            hex::decode(pbk_str.as_ref()).expect("Failed to parse the public key from hexadecimal");

        let app_public_key =
            PublicKey::from_bytes(&pbk_bytes).expect("Failed to parse public key.");

        if let Some(token) = token {
            let mut headers = header::HeaderMap::new();

            // Let it panic if there is no valid value
            let auth_value = header::HeaderValue::from_str(token.as_str());

            assert!(!auth_value.is_err(), "Invalid value given at token");

            let mut auth_value = auth_value.unwrap();

            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
            let new_c = Client::builder().default_headers(headers).build().unwrap();

            InteractionHandler {
                application_id: app_id,
                app_public_key,
                client: new_c,
                global_handles: HashMap::new(),
                component_handles: HashMap::new(),
                guild_handles: HashMap::new(),
                data: AnyMap::new(),
            }
        } else {
            InteractionHandler {
                application_id: app_id,
                app_public_key,
                client: Client::new(),
                global_handles: HashMap::new(),
                component_handles: HashMap::new(),
                guild_handles: HashMap::new(),
                data: AnyMap::new(),
            }
        }
    }

    /// Add some data. Data can be accessed by handlers with `InteractionHandler.data`
    pub fn add_data<T: Clone>(&mut self, data: T)
    where
        T: Send + 'static + Sync,
    {
        self.data.insert(data);
    }
    /// Binds an async function to a **global** command.
    /// Your function must take a [`Context`] and optionally an [`InteractionHandler`] as an argument and must return a [`InteractionResponse`].
    /// Make sure to use the `#[slash_command]` procedural macro to make it usable for the handler.
    ///
    /// Like:
    /// ```ignore
    /// # use rusty_interaction::types::interaction::{Context, InteractionResponse};
    /// # use attributes::slash_command;
    /// #[slash_command]
    /// async fn do_work(ctx: Context) -> InteractionResponse {
    ///     return todo!("Do work and return a response");
    /// }
    /// ```
    ///
    /// # Note
    /// The handler will first check if a guild-specific handler is available. If not, it will try to match a global command. If that fails too, an error will be returned.
    ///
    /// # Example
    /// ```ignore
    /// # use rusty_interaction::types::interaction::{Context, InteractionResponse};
    /// # use rusty_interaction::handler::InteractionHandler;
    /// # use attributes::slash_command;
    /// const PUB_KEY: &str = "my_public_key";
    ///
    /// #[slash_command]
    /// async fn pong_handler(ctx: Context) -> InteractionResponse {
    ///     return ctx.respond()
    ///             .content("Pong!")
    ///             .build()
    ///             .unwrap();
    /// }
    ///
    /// #[actix_web::main]
    /// async fn main() -> std::io::Result<()> {
    ///
    ///     let mut handle = InteractionHandler::new(PUB_KEY);
    ///     handle.add_command("ping", pong_handler);
    ///
    ///     return handle.run().await;
    /// }
    /// ```
    pub fn add_global_command(&mut self, name: &'static str, func: HandlerFunction) {
        self.global_handles.insert(name, func);
    }

    /// Binds an async function to a **component**.
    /// Your function must take a [`Context`] and optionally an [`InteractionHandler`] as an argument and must return a [`InteractionResponse`].
    /// Use the `#[component_handler]` procedural macro for your own convinence.eprintln!
    ///
    /// # Example
    /// ```ignore
    /// use rusty_interaction::handler::InteractionHandler;
    /// use rusty_interaction::types::components::*;
    /// use rusty_interaction::types::interaction::*;
    ///
    /// #[component_handler]
    /// async fn comp_hand(ctx: Context) -> InteractionResponse {
    ///     return ctx.respond().content("Some message content").build();
    /// }
    ///
    /// #[slash_command]
    /// async fn spawn_buttons(ctx: Context) -> InteractionResponse {
    ///     // Let's build our message!
    ///     let resp = ctx.respond()
    ///     // Set message content
    ///     .content("Not edited")
    ///     // add a component action row using it's builder
    ///     .add_component_row(
    ///         ComponentRowBuilder::default()
    ///        // Add buttons using it's builder
    ///        .add_button(
    ///           ComponentButtonBuilder::default()
    ///                            .label("Edit")
    ///                            .custom_id("HEHE")
    ///                            .style(ComponentButtonStyle::Primary)
    ///                            .build()
    ///                            .unwrap()
    ///        )
    ///        .add_button(
    ///            ComponentButtonBuilder::default()
    ///                            .label("Delete")
    ///                            .custom_id("DELETE")
    ///                            .style(ComponentButtonStyle::Danger)
    ///                            .build()
    ///                            .unwrap()
    ///         )
    ///         .build()
    ///         .unwrap()
    ///     )
    ///     .build()
    ///     .unwrap();

    ///     return resp;
    ///
    /// }
    /// #[actix_web::main]
    /// async fn main() -> std::io::Result<()> {
    ///
    ///     let mut handle = InteractionHandler::new(PUB_KEY);
    ///     handle.add_command("ping", pong_handler);
    ///     handle.add_component_handle("HEHE", comp_hand);
    ///     return handle.run().await;
    /// }
    /// ```
    pub fn add_component_handle(&mut self, custom_id: &'static str, func: HandlerFunction) {
        self.component_handles.insert(custom_id, func);
    }

    #[cfg(feature = "extended-handler")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-handler")))]
    /// Register a guild-specific command with Discord!
    ///
    /// # NOTE
    /// Guild-specific commands are not cached or saved in any way by the handler.
    /// This means that between restarts, updates, crashes, or whatever that causes the app to terminate, the handler 'forgets' which command belonged to which handler.
    pub async fn register_guild_handle(
        &mut self,
        guild: impl Into<Snowflake>,
        cmd: &ApplicationCommand,
        func: HandlerFunction,
        scope: &ManipulationScope,
    ) -> Result<ApplicationCommand, HttpError> {
        let g = guild.into();
        match scope {
            ManipulationScope::Local => {
                self.guild_handles.insert(g, func);
                Ok(cmd.clone())
            }
            ManipulationScope::Discord | ManipulationScope::All => {
                let url = format!(
                    "{}/applications/{}/guilds/{}/commands",
                    crate::BASE_URL,
                    self.application_id,
                    g
                );

                let r = self.client.post(&url).json(cmd).send().await;

                expect_successful_api_response_and_return!(r, ApplicationCommand, a, {
                    if let Some(id) = a.id {
                        if scope == &ManipulationScope::All {
                            // Already overwrites current key if it exists, so no need to check.
                            self.guild_handles.insert(id, func);
                        }

                        Ok(a)
                    } else {
                        // Pretty bad if this code reaches...
                        Err(HttpError {
                            code: 0,
                            message: "Command registration response did not have an ID."
                                .to_string(),
                        })
                    }
                })
            }
        }
    }

    #[cfg(feature = "extended-handler")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-handler")))]
    /// Remove a guild handle
    pub async fn deregister_guild_handle(
        &mut self,
        guild: impl Into<Snowflake>,
        id: impl Into<Snowflake>,
        scope: &ManipulationScope,
    ) -> Result<(), HttpError> {
        let i = id.into();
        let g = guild.into();

        match scope {
            ManipulationScope::Local => {
                self.guild_handles.remove(&i);
                Ok(())
            }
            ManipulationScope::All | ManipulationScope::Discord => {
                let url = format!(
                    "{}/applications/{}/guilds/{}/commands/{}",
                    crate::BASE_URL,
                    self.application_id,
                    g,
                    i
                );

                let r = self.client.delete(&url).send().await;

                expect_specific_api_response!(r, StatusCode::NO_CONTENT, {
                    if scope == &ManipulationScope::All {
                        self.guild_handles.remove(&i);
                    }

                    Ok(())
                })
            }
        }
    }

    #[cfg(feature = "extended-handler")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-handler")))]
    /// Override a bunch of permissions for commands in a guild.
    pub async fn override_guild_permissions(
        &self,
        guild_id: impl Into<Snowflake>,
        overrides: &[ApplicationCommandPermissionBatch],
    ) -> Result<(), HttpError> {
        let url = format!(
            "{}/applications/{}/guilds/{}/commands/permissions",
            crate::BASE_URL,
            self.application_id,
            guild_id.into()
        );

        let res = self.client.put(&url).json(overrides).send().await;

        expect_successful_api_response!(res, Ok(()))
    }

    #[cfg(feature = "extended-handler")]
    #[cfg_attr(docsrs, doc(cfg(feature = "extended-handler")))]
    /// Add a permission override for a guild command
    pub async fn edit_guild_command_permissions(
        &self,
        guild_id: impl Into<Snowflake>,
        appcmd_id: impl Into<Snowflake>,
        permission_override: &ApplicationCommandPermission,
    ) -> Result<(), HttpError> {
        let url = format!(
            "{}/applications/{}/guilds/{}/commands/{}/permissions",
            crate::BASE_URL,
            self.application_id,
            guild_id.into(),
            appcmd_id.into(),
        );

        let res = self.client.put(&url).json(permission_override).send().await;

        expect_successful_api_response!(res, Ok(()))
    }

    /// Entry point function for handling `Interactions`
    pub async fn interaction(&mut self, req: HttpRequest, body: String) -> Result<HttpResponse> {
        // Check for good content type --> must be application/json

        if let Some(ct) = req.headers().get("Content-Type") {
            if ct != "application/json" {
                debug!(
                    "Incoming interaction rejected, bad Content-Type specified. Origin: {:?}",
                    req.connection_info().realip_remote_addr()
                );
                return ERROR_RESPONSE!(400, "Bad Content-Type");
            }
        } else {
            debug!(
                "Incoming interaction rejected, no Content-Type specified. Origin: {:?}",
                req.connection_info().realip_remote_addr()
            );
            return ERROR_RESPONSE!(400, "Bad Content-Type");
        }

        let se = get_header(&req, "X-Signature-Ed25519");
        let st = get_header(&req, "X-Signature-Timestamp");

        // TODO: Domain check might be a good one.

        if let Some((se, st)) = se.zip(st) {
            // Verify timestamp + body against given signature
            if !verify_discord_message(self.app_public_key, se, st, &body).is_ok() {
                // Verification failed, reject.
                // TODO: Switch error response
                debug!(
                    "Incoming interaction rejected, invalid signature. Origin: {:?}",
                    req.connection_info().realip_remote_addr()
                );
                return ERROR_RESPONSE!(401, "Invalid request signature");
            }
        } else {
            // If proper headers are not present reject.
            debug!(
                "Incoming interaction rejected, missing headers. Origin: {:?}",
                req.connection_info().realip_remote_addr()
            );
            return ERROR_RESPONSE!(400, "Bad signature data");
        }

        // Security checks passed, try deserializing request to Interaction.
        match serde_json::from_str::<Interaction>(&body) {
            Err(e) => {
                // It's probably bad on our end if this code is reached.
                // Could be that new features were implemented and breaks code
                error!("Failed to decode interaction! Error: {}", e);
                debug!("Body sent: {}", body);
                return ERROR_RESPONSE!(400, format!("Bad body: {}", e));
            }
            Ok(interaction) => {
                match interaction.r#type {
                    InteractionType::Ping => {
                        let response =
                            InteractionResponse::new(InteractionResponseType::Pong, None);
                        debug!("Got a ping, responding with pong.");
                        return Ok(HttpResponse::build(StatusCode::OK)
                            .content_type("application/json")
                            .json(response));
                    }

                    InteractionType::ApplicationCommand => {
                        let data = if let Some(ref data) = interaction.data {
                            data
                        } else {
                            error!("Failed to unwrap Interaction!");
                            return ERROR_RESPONSE!(500, "Failed to unwrap");
                        };

                        // Check for matches in guild handler map. Unwrapping because this should always contain an ID
                        if let Some(handler) = self.guild_handles.get(data.id.as_ref().unwrap()) {
                            // construct a Context
                            let ctx = Context::new(self.client.clone(), interaction);

                            // Call the handler
                            let response = handler(self, ctx).await;

                            match_handler_response!(response)
                        }
                        // Welp, nothing found. Check for matches in the global map
                        else if let Some(handler) = self.global_handles.get(
                            data.name.as_ref().unwrap().as_str(), /* Don't question it */
                        ) {
                            // construct a Context
                            let ctx = Context::new(self.client.clone(), interaction);

                            // Call the handler
                            let response = handler(self, ctx).await;

                            match_handler_response!(response)
                        }
                        // Still nothing, return an error
                        else {
                            error!(
                                "No associated handler found for {}",
                                data.name.as_ref().unwrap().as_str()
                            );
                            ERROR_RESPONSE!(501, "No associated handler found")
                        }
                    }
                    InteractionType::MessageComponent => {
                        let data = if let Some(ref data) = interaction.data {
                            data
                        } else {
                            error!("Failed to unwrap Interaction!");
                            return ERROR_RESPONSE!(500, "Failed to unwrap");
                        };

                        if let Some(handler) = self
                            .component_handles
                            .get(data.custom_id.as_ref().unwrap().as_str())
                        {
                            // construct a Context
                            let ctx = Context::new(self.client.clone(), interaction);

                            // Call the handler
                            let response = handler(self, ctx).await;

                            match_handler_response!(response)
                        } else {
                            error!(
                                "No associated handler found for {}",
                                data.custom_id.as_ref().unwrap().as_str()
                            );
                            ERROR_RESPONSE!(501, "No associated handler found")
                        }
                    }
                    InteractionType::ApplicationCommandAutocomplete => {
                        //TODO: Implement autocomplete stuff
                        unimplemented!();
                    }
                    InteractionType::ModalSubmit => {
                        //TODO: Implement Modal submit event
                        unimplemented!();
                    }
                }
            }
        }
    }

    /// This is a predefined function that starts an `actix_web::HttpServer` and binds `self.interaction` to `/api/discord/interactions`.
    /// Note that you'll eventually have to switch to an HTTPS server. This function does not provide this.
    pub async fn run(self, port: u16) -> std::io::Result<()> {
        let data = web::Data::new(Mutex::new(self));
        HttpServer::new(move || {
            App::new().app_data(data.clone()).route(
                "/api/discord/interactions",
                web::post().to(
                    |data: web::Data<Mutex<InteractionHandler>>, req: HttpRequest, body: String| async move {
                        data.lock().unwrap().interaction(req, body).await
                    },
                ),
            )
        })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
    }

    /// Same as [`InteractionHandler::run`] but starts a server with SSL/TLS.
    pub async fn run_ssl(self, server_conf: ServerConfig, port: u16) -> std::io::Result<()> {
        let data = web::Data::new(Mutex::new(self));
        HttpServer::new(move || {
            App::new().app_data(data.clone()).route(
                "/api/discord/interactions",
                web::post().to(
                    |data: web::Data<Mutex<InteractionHandler>>, req: HttpRequest, body: String| async move {
                        data.lock().unwrap().interaction(req, body).await
                    },
                ),
            )
        })
        .bind_rustls(format!("0.0.0.0:{}", port), server_conf)?
        .run()
        .await
    }
}

/// Simpler header getter from a HTTP request
fn get_header<'a>(req: &'a HttpRequest, header: &str) -> Option<&'a str> {
    req.headers().get(header)?.to_str().ok()
}
