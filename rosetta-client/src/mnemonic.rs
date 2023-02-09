use crate::crypto::bip39::{Language, Mnemonic};
use anyhow::Result;
#[cfg(not(target_family = "wasm"))]
use std::fs::OpenOptions;
#[cfg(not(target_family = "wasm"))]
use std::io::Write;
use std::path::Path;
#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsCast, UnwrapThrowExt};
#[cfg(target_family = "wasm")]
use web_sys::Storage;

pub fn generate_mnemonic() -> Result<Mnemonic> {
    let mut entropy = [0; 32];
    getrandom::getrandom(&mut entropy)?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    Ok(mnemonic)
}

pub struct MnemonicStore {
    #[cfg(not(target_family = "wasm"))]
    path: PathBuf,
    #[cfg(target_family = "wasm")]
    local_storage: Storage,
}

impl MnemonicStore {
    pub fn generate(&self) -> Result<Mnemonic> {
        let mnemonic = generate_mnemonic()?;
        self.set(&mnemonic)?;
        Ok(mnemonic)
    }

    pub fn get_or_generate_mnemonic(&self) -> Result<Mnemonic> {
        if self.exists() {
            self.get()
        } else {
            self.generate()
        }
    }
}

#[cfg(not(target_family = "wasm"))]
impl MnemonicStore {
    pub fn new(path: Option<&Path>) -> Result<Self> {
        let path = if let Some(path) = path {
            path.into()
        } else {
            dirs_next::config_dir()
                .ok_or_else(|| anyhow::anyhow!("no config dir found"))?
                .join("rosetta-wallet")
                .join("mnemonic")
        };
        Ok(Self { path })
    }

    pub fn set(&self, mnemonic: &Mnemonic) -> Result<()> {
        std::fs::create_dir_all(self.path.parent().unwrap())?;
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = OpenOptions::new();
        opts.create(true).write(true).truncate(true);
        #[cfg(unix)]
        opts.mode(0o600);
        let mut f = opts.open(&self.path)?;
        f.write_all(mnemonic.to_string().as_bytes())?;
        Ok(())
    }

    pub fn get(&self) -> Result<Mnemonic> {
        let mnemonic = std::fs::read_to_string(&self.path)?;
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
        Ok(mnemonic)
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(target_family = "wasm")]
impl MnemonicStore {
    pub fn new(_path: Option<&Path>) -> Result<Self> {
        let local_storage = web_sys::window()
            .expect_throw("no window")
            .local_storage()
            .expect_throw("failed to get local_storage")
            .expect_throw("no local storage");
        Ok(Self { local_storage })
    }

    pub fn set(&self, mnemonic: &Mnemonic) -> Result<()> {
        self.local_storage
            .set_item("mnemonic", &mnemonic.to_string())
            .map_err(|value| {
                anyhow::anyhow!(String::from(
                    value.dyn_into::<js_sys::Error>().unwrap().to_string()
                ))
            })?;
        Ok(())
    }

    pub fn get(&self) -> Result<Mnemonic> {
        let mnemonic = self
            .local_storage
            .get_item("mnemonic")
            .expect_throw("unreachable: get_item does not throw an exception")
            .expect_throw("no mnemonic in store");
        Ok(Mnemonic::parse_in(Language::English, &mnemonic)?)
    }

    pub fn exists(&self) -> bool {
        self.local_storage
            .get_item("mnemonic")
            .expect_throw("unreachable: get_item does not throw an exception")
            .is_some()
    }
}
