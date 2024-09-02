use std::{
    fs::{create_dir, File},
    path::Path,
};

use color_eyre::eyre::WrapErr;
use judge::submit::*;
use schemars::schema_for;

fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;

    let schema_path = Path::new("schema");
    if !schema_path.is_dir() {
        create_dir(&schema_path).wrap_err("failed to create schema directory")?;
    }

    for (filename, schema) in [
        ("report.json", schema_for!(Report)),
        ("test_report.json", schema_for!(TestReport)),
        ("verdict.json", schema_for!(Verdict)),
        ("message.json", schema_for!(Message)),
    ] {
        let file =
            File::create(schema_path.join(filename)).wrap_err("failed to create schema file")?;
        serde_json::to_writer_pretty(file, &schema).wrap_err("failed to serialize JSON schema")?;
    }

    println!("generated JSON schema files in ./schema");
    println!("move generated files to the frontend package root");
    Ok(())
}
