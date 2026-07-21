pub mod log;
pub mod graph;
pub mod util;
pub mod init;
pub mod package;

pub const BUILD_FOLDER: &str = "build/";
pub const BUILD_FILE: &str = "dist.ospb";

pub fn load_package_cfg<P: AsRef<std::path::Path>>(path: P) -> graph::decl::PackageSetup {
    let src = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .expect("DAMN IT I CANT LOAD PKG.KDL");

    let k = kdl::KdlDocument::parse(&src).expect("failed to parse package.kdl");

    return package::parse_kdl(k).expect("failed to parse the KDL");
}