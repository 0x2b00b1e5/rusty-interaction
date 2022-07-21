#[macro_use]
extern crate rusty_interaction;

use rusty_interaction::handler::{InteractionHandler, ManipulationScope};
use rusty_interaction::types::components::*;
use rusty_interaction::types::interaction::*;
// Relevant imports here
use rusty_interaction::types::modal::{Modal, ModalBuilder};

use rusty_interaction::Builder;

const PUB_KEY: &str = "57028473720a7c1d4666132a68007f0902034a13c43cc2c1658b10b5fc754311";
const APP_ID: u64 = 615112470033596416;

#[slash_command]
async fn test(
    handler: &mut InteractionHandler,
    ctx: Context,
) -> Result<InteractionResponse, std::convert::Infallible> {
    println!("Got trigger");
    let test_modal = ModalBuilder::default()
        .custom_id("TEST_MODAL")
        .title("My Test Modal")
        .add_component(
            ComponentTextBoxBuilder::default()
                .placeholder("Some placeholder") 
                .max_length(100) // Sets a maximum of 100 chars
                .label("My label")
                .custom_id("MODAL_TEXT_BOX")
                .required(true)
                .style(ComponentTextBoxStyle::Paragraph) // Sets the textbox to be a paragraph style textbox (multiline)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    Ok(ctx.respond_with_modal(test_modal))
}

// The lib uses actix-web
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut handle = InteractionHandler::new(APP_ID, PUB_KEY, None);

    handle.add_global_command("summon", test);

    return handle.run(10080).await;
}
