//! File format identification

use file_format::FileFormat;
use tracing::warn;

use crate::module::data::ModuleDataKind;

impl From<FileFormat> for ModuleDataKind {
    fn from(format: FileFormat) -> Self {
        match format.kind() {
            file_format::Kind::Image => {
                if FileFormat::ScalableVectorGraphics == format {
                    ModuleDataKind::Svg
                } else {
                    ModuleDataKind::Image
                }
            }
            file_format::Kind::Font => todo!(),
            file_format::Kind::Other => {
                if FileFormat::PlainText == format {
                    ModuleDataKind::Text
                } else {
                    warn!("Unhandled file kind: {:?}", format.kind());
                    ModuleDataKind::Unknown
                }
            }

            _ => {
                warn!("Unhandled file kind: {:?}", format.kind());
                ModuleDataKind::Unknown
            }
        }
    }
}
