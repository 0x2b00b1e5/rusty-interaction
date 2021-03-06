# Example 1: A basic handler
This the most basic handler you can make with Rusty Interaction. 

If `/summon` was called, it will print `I HAVE BEEN SUMMONED!!!` on the console and reply with `I was summoned?`.

## Important design note
Whatever you return is **the initial response**. It was chosen this way because Discord _always_ wants a response from you. This way, you're forced to
give a response.

However, this might be confusing at times. Especially if you're using followup messages and/or you're editing your original response. Be aware of this.

# Running this example
You can use regular `cargo build` and `cargo run` commands.

To run this example:
`cargo run`. Note that you'll need to edit the `PUB_KEY` constant accordingly (it will panic if you don't give a vaild key).

# Security
This example starts a plain HTTP server. Using plain HTTP these days is a **bad idea**. 

Look at example 2 for a HTTPS server implementation.

# Docs to read
- [InteractionHandler](https://docs.rs/rusty_interaction/latest/rusty_interaction/handler/struct.InteractionHandler.html)
- [Context](https://docs.rs/rusty_interaction/0.1.0/rusty_interaction/types/interaction/struct.Context.html)