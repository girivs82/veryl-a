use crate::OptNew;
use log::info;
use miette::{bail, IntoDiagnostic, Result};
use std::fs::{self, File};
use std::io::Write;
use veryla_metadata::Metadata;

pub struct CmdNew {
    opt: OptNew,
}

impl CmdNew {
    pub fn new(opt: OptNew) -> Self {
        Self { opt }
    }

    pub fn exec(&self) -> Result<bool> {
        if self.opt.path.exists() {
            bail!("path \"{}\" exists", self.opt.path.to_string_lossy());
        }

        if let Some(name) = self.opt.path.file_name() {
            let name = name.to_string_lossy();
            let toml = Metadata::create_default_toml(&name).into_diagnostic()?;

            fs::create_dir_all(&self.opt.path).into_diagnostic()?;
            let mut file = File::create(self.opt.path.join("Veryla.toml")).into_diagnostic()?;
            write!(file, "{toml}").into_diagnostic()?;
            file.flush().into_diagnostic()?;

            info!("Created \"{}\" project", name);
        } else {
            bail!("path \"{}\" is not valid", self.opt.path.to_string_lossy());
        }

        Ok(true)
    }
}
