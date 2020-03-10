use std::path::PathBuf;

#[derive(Debug)]
pub struct GitRepo {
    pub remote_url: String,
    pub local_path: PathBuf,
}
