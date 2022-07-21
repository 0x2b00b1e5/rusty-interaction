# Example 9: Accessing other data

From version 0.2.0, the `InteractionHandler` has a field called `data`. This is used to access other data, like database connections for example.

You can add data to the handler using `InteractionHandler::add_data()`. The backbone is an `AnyMap` and shares the same syntax with accessing data.


# Result
![Peek 2021-07-29 21-53](https://user-images.githubusercontent.com/10338882/127557511-724e139a-4a5c-44cf-b403-6d270bbd8953.gif)


# Running this example
You can use regular `cargo build` and `cargo run` commands.

To run this example:

`cargo run`. Note that you'll need to edit the `PUB_KEY`, `APP_ID` and `TOKEN` constants accordingly (it will panic if you don't give a vaild key).

# Useful documentation
- [InteractionHandler](https://docs.rs/rusty_interaction/latest/rusty_interaction/handler/struct.InteractionHandler.html)
