use std::process::Command;
use crate::package::package::PackageInfo;

pub fn retrieve(path: &str) -> Result<Vec<PackageInfo>, String> {
    let output_res = Command::new("dpkg-query")
        .args(["-S", path])
        .output()
        .map_err(|e| e.to_string());
    match output_res {
        Err(reason) => return Err(reason),
        _ => ()
    }
    let output = output_res.unwrap();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<String> = stdout
        .lines()
        .filter_map(|line| line.split(':').next())
        .map(str::trim)
        .map(|s| s.to_string())
        .collect();
    let mut packages_obj = Vec::new();
    for package in packages {
        let v = version(package.as_str());
        let vstr = match v {
            None => String::from("cannot retrieve the version"),
            Some(vstrretrieved) => vstrretrieved
        };
        packages_obj.push(PackageInfo {
            name: package,
            version: vstr
        });
    }
    return Ok(packages_obj);
}

fn version(package: &str) -> Option<String> {
    let out = Command::new("dpkg-query")
        .args(["-W", "-f=${Version}", package])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    return Some(s);
}
