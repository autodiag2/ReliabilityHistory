use std::path::Path;
use crate::package::deb as deb;

pub struct PackageDB {
    backend: &'static str
}

#[derive(serde::Serialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String
}

impl PackageDB {
    pub fn new() -> Self {
        if Path::new("/usr/bin/dpkg-query").exists() {
            PackageDB { backend: "deb" }
        } else if Path::new("/usr/bin/rpm").exists() {
            PackageDB { backend: "rpm" }
        } else if Path::new("/usr/bin/pacman").exists() {
            PackageDB { backend: "pacman" }
        } else {
            PackageDB { backend: "unknown" }
        }
    }

    pub fn retrieve(&self, path: &str) -> Option<Vec<PackageInfo>> {
        return match self.backend {
            "deb" => match deb::retrieve(path) {
                Err(_) => None,
                Ok(packages) => Some(packages)
            },
            _ => None
        }
    }

}