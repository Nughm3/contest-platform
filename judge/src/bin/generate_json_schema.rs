use std::fs::File;

use color_eyre::eyre::WrapErr;
use judge::submit::*;
use schemars::schema_for;

fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;

    let file = File::create("schema.json").wrap_err("failed to create schema file")?;
    serde_json::to_writer_pretty(file, &schema_for!(Message))
        .wrap_err("failed to serialize JSON schema")?;
    println!("generated schema.json");

    Ok(())
}
