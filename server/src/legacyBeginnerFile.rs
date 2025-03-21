// #[spacetimedb::reducer]
// pub fn say_hello(ctx: &ReducerContext) {
//     for person in ctx.db.person().iter() {
//         log::info!("Hello, {}!", person.name);
//     }
//     log::info!("Hello, World!");
// }

// #[reducer]
// /// Clients invoke this reducer to set their user names.
// pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
//     let name = validate_name(name)?;
//     if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
//         ctx.db.user().identity().update(User {
//             name: Some(name),
//             ..user
//         });
//         Ok(())
//     } else {
//         Err("Cannot set name for unknown user".to_string())
//     }
// }

// // / Takes a name and checks if it's acceptable as a user's name.
// fn validate_name(name: String) -> Result<String, String> {
//     if name.is_empty() {
//         Err("Names must not be empty".to_string())
//     } else {
//         Ok(name)
//     }
// }

// #[reducer]
// /// Clients invoke this reducer to send messages.
// pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
//     let text = validate_message(text)?;

//     log::info!("{}", text);
//     ctx.db.message().insert(Message {
//         sender: ctx.sender,
//         text,
//         sent: ctx.timestamp,
//     });
//     Ok(())
// }

// /// Takes a message's text and checks if it's acceptable to send.
// fn validate_message(text: String) -> Result<String, String> {
//     if text.is_empty() {
//         Err("Messages must not be empty".to_string())
//     } else {
//         Ok(text)
//     }
// }
