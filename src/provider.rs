use crate::dep::Dep;
use crate::error::XbpsError;

pub trait PackageProvider: Sync {
    fn deps(&self, pkg: &str) -> Result<Vec<Dep>, XbpsError>;
    fn rdeps(&self, pkg: &str) -> Result<Vec<Dep>, XbpsError>;
    fn version(&self, pkg: &str) -> Result<Option<String>, XbpsError>;
}
