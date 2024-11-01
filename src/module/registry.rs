use std::str::FromStr;

use strum::EnumString;

use super::{error::ModuleError, manager::ModuleManager, HandleId};

#[derive(EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ModuleKind {
    Timing,
    Http,
}

impl ModuleManager {
    pub fn create(&mut self, kind: ModuleKind) -> HandleId {
        match kind {
            ModuleKind::Timing => self.create_inner::<super::timing::TimingModule>(),
            ModuleKind::Http => self.create_inner::<super::http::HttpModule>(),
        }
    }

    pub fn from_string(name: String) -> Result<ModuleKind, ModuleError> {
        match ModuleKind::from_str(name.as_str()) {
            Ok(kind) => Ok(kind),
            Err(_e) => Err(ModuleError::Unknown(name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module::manager::ModuleManager;

    #[test]
    fn from_string() {
        let res = ModuleManager::from_string("notexist".into());
        assert!(res.is_err());

        let res = ModuleManager::from_string("timing".into());
        assert!(res.is_ok());

        let res = ModuleManager::from_string("Timing".into());
        assert!(res.is_ok());

        let res = ModuleManager::from_string("TIMING".into());
        assert!(res.is_ok());
    }
}
