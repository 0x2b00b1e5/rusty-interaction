use crate::{expect_successful_api_response_and_return};
use crate::security::*;
use crate::types::HttpError;
use crate::types::interaction::*;
use crate::types::Snowflake;
use crate::types::application::*;
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use reqwest::header;
use reqwest::Client;

use std::fmt;
use log::{debug, error};

use ed25519_dalek::PublicKey;

use rustls::ServerConfig;

use std::{collections::HashMap, future::Future, pin::Pin, sync::Mutex};

/// Alias for InteractionResponse
pub type HandlerResponse = InteractionResponse;

type HandlerFunction = fn(&mut InteractionHandler, Context) -> Pin<Box<dyn Future<Output = HandlerResponse> + Send + '_>>;



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
    guild_handles: HashMap<Snowflake, HandlerFunction>
}

// Only here to make Debug less generic, so I can send a reference 
impl fmt::Debug for InteractionHandler{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        return f.debug_struct("InteractionHandler")
            .field("app_public_key", &self.app_public_key)
            .field("global_handles_len", &self.global_handles.len())
            .field("component_handles_len", &self.component_handles.len())
            .finish();
    }
}

impl InteractionHandler {
    /// Initalizes a new `InteractionHandler`
    pub fn new(app_id: Snowflake, pbk_str: &str, token: Option<&'static str>) -> InteractionHandler {
        let pbk_bytes =
            hex::decode(pbk_str).expect("Failed to parse the public key from hexadecimal");

        let app_public_key =
            PublicKey::from_bytes(&pbk_bytes).expect("Failed to parse public key.");

        if let Some(token) = token{  
            let mut headers = header::HeaderMap::new();
            let mut auth_value = header::HeaderValue::from_static(token);
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
            let new_c = Client::builder().default_headers(headers).build().unwrap();

            InteractionHandler {
                application_id: app_id,
                app_public_key: app_public_key,
                client: new_c,
                global_handles: HashMap::new(),
                component_handles: HashMap::new(),
                guild_handles: HashMap::new(),
            }
        }
        else{
            InteractionHandler {
                application_id: app_id,
                app_public_key: app_public_key,
                client: Client::new(),
                global_handles: HashMap::new(),
                component_handles: HashMap::new(),
                guild_handles: HashMap::new(),
            }
        }

        
    }

    /// Binds an async function to a **global** command.
    /// Your function must take a [`Context`] as an argument and must return a [`InteractionResponse`].
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
    ///             .finish();
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
    /// Your function must take a [`Context`] as an argument and must return a [`InteractionResponse`].
    /// Use the `#[component_handler]` procedural macro for your own convinence.eprintln!
    /// 
    /// Example
    /// ```ignore
    /// use rusty_interaction::handler::InteractionHandler;
    /// use rusty_interaction::types::components::*;
    /// use rusty_interaction::types::interaction::*;
    /// 
    /// #[component_handler]
    /// async fn comp_hand(ctx: Context) -> InteractionResponse {
    ///     return ctx.respond().content("Some message content").finish();
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
    ///                            .finish()
    ///        )
    ///        .add_button(
    ///            ComponentButtonBuilder::default()
    ///                            .label("Delete")
    ///                            .custom_id("DELETE")
    ///                            .style(ComponentButtonStyle::Danger)
    ///                            .finish()
    ///         )
    ///         .finish()
    ///     )
    ///     .finish();

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

    /// Register an command with Discord!
    pub async fn register_command_handle(&mut self, guild: impl Into<Snowflake>, cmd: ApplicationCommand, func: HandlerFunction) -> Result<ApplicationCommand, HttpError>{
        let g = guild.into();

        let url = format!("{}/applications/{}/guilds/{}/commands", crate::BASE_URL, self.application_id, g);

        let r = self.client.post(&url).json(&cmd).send().await;

        expect_successful_api_response_and_return!(r, ApplicationCommand, a, {
            if let Some(id) = a.id{
                self.guild_handles.insert(id, func);
                Ok(a)
            }
            else{
                // Pretty bad if this code reaches...
                Err(HttpError{
                    code: 0,
                    message: "Command registration response did not have an ID.".to_string(),
                })
            }
        })

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
            if verify_discord_message(self.app_public_key, se, st, &body).is_ok() {
                // Signature OK. Continue with interaction processing.
            } else {
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

                        if let Some(handler) = self
                            .global_handles
                            .get(data.name.as_ref().unwrap().as_str())
                        {
                            // do stuff with v if needed

                            // construct a Context
                            let ctx = Context::new(self.client.clone(), interaction);

                            // Call the handler
                            let response = handler(self, ctx).await;

                            match response.r#type {
                                InteractionResponseType::None => {
                                    Ok(HttpResponse::build(StatusCode::NO_CONTENT).finish())
                                }
                                InteractionResponseType::DefferedChannelMessageWithSource
                                | InteractionResponseType::DefferedUpdateMessage => {
                                    /* The use of HTTP code 202 is more appropriate when an Interaction is deffered.
                                    If an application is first sending a deffered channel message response, this usually means the system
                                    is still processing whatever it is doing.
                                    See the spec: https://tools.ietf.org/html/rfc7231#section-6.3.3 */
                                    Ok(HttpResponse::build(StatusCode::ACCEPTED).json(response))
                                }
                                _ => {
                                    // Send out a response to Discord
                                    let r = HttpResponse::build(StatusCode::OK).json(response);

                                    Ok(r)
                                }
                            }
                        } else {
                            error!(
                                "No associated handler found for {}",
                                data.name.as_ref().unwrap().as_str()
                            );
                            ERROR_RESPONSE!(500, "No associated handler found")
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

                            if response.r#type == InteractionResponseType::DefferedUpdateMessage {
                                /* The use of HTTP code 202 is more appropriate when an Interaction is deffered.
                                If an application is first sending a deffered channel message response, this usually means the system
                                is still processing whatever it is doing.
                                See the spec: https://tools.ietf.org/html/rfc7231#section-6.3.3 */
                                Ok(HttpResponse::build(StatusCode::ACCEPTED).json(response))
                            } else {
                                // Send out a response to Discord
                                let r = HttpResponse::build(StatusCode::OK).json(response);

                                Ok(r)
                            }
                        } else {
                            error!(
                                "No associated handler found for {}",
                                data.name.as_ref().unwrap().as_str()
                            );
                            ERROR_RESPONSE!(500, "No associated handler found")
                        }
                    }
                }
            }
        }
    }

    /// This is a predefined function that starts an `actix_web::HttpServer` and binds `self.interaction` to `/api/discord/interactions`.
    /// Note that you'll eventually have to switch to an HTTPS server. This function does not provide this.
    ///
    /// **The server runs on port 10080!**
    pub async fn run(self) -> std::io::Result<()> {
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
        .bind("0.0.0.0:10080")?
        .run()
        .await
    }

    /// Same as [`InteractionHandler::run`] but starts a server with SSL/TLS.
    ///
    /// **The server runs on port 10443!**
    pub async fn run_ssl(self, server_conf: ServerConfig) -> std::io::Result<()> {
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
        .bind_rustls("0.0.0.0:10443", server_conf)?
        .run()
        .await
    }
}

/// Simpler header getter from a HTTP request
fn get_header<'a>(req: &'a HttpRequest, header: &str) -> Option<&'a str> {
    req.headers().get(header)?.to_str().ok()
}
