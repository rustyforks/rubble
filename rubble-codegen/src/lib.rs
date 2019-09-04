//! Code generator for use in `build.rs`.

use std::{
    io::prelude::*,
    fs::File,
    env,
    path::PathBuf,
    error::Error,
};

pub type BoxedError = Box<dyn Error + Send + Sync>;

/// Builder for attribute sets.
#[derive(Default)]
pub struct Builder {}

impl Builder {
    /// Creates a new builder that will produce a minimal GATT server.
    ///
    /// The minimal GATT server contains only a GAP service, which is mandatory for BLE devices.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates Rust code and writes it to a file in the target directory.
    ///
    /// The file can be included into the main crate by calling the macro
    /// `rubble::include_attributes!`.
    pub fn build(self) {
        self.try_build().unwrap()
    }

    pub fn try_build(self) -> Result<(), BoxedError> {
        let mut path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        path.push("rubble_codegen.rs");
        let mut file = File::create(path)?;
        writeln!(file, "oops")?;

        println!("cargo:rerun-if-changed=build.rs");
        Ok(())
    }
}
