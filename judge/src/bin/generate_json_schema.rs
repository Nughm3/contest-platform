use std::fs;

use color_eyre::eyre::WrapErr;
use judge::{contest::Contest, submit::Message};
use schemars::schema_for;

fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;

    let contest = serde_json::to_string_pretty(&schema_for!(Contest))
        .wrap_err("failed to serialize JSON schema for Contest")?;
    fs::write("schema/contest.json", contest).wrap_err("failed to write schema/contest.json")?;
    println!("generated schema/contest.json");

    let message = serde_json::to_string_pretty(&schema_for!(Message))
        .wrap_err("failed to serialize JSON schema for Message")?;
    fs::write("schema/message.json", message).wrap_err("failed to write schema/message.json")?;
    println!("generated schema/message.json");

    Ok(())
}
