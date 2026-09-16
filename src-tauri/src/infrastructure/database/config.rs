use std::path::PathBuf;

#[cfg(not(test))]
pub const BCRYPT_COST: u32 = 10;
#[cfg(test)]
pub const BCRYPT_COST: u32 = 4;

#[cfg(test)]
pub fn get_db_path() -> PathBuf {
    PathBuf::from(":memory:")
}

#[cfg(not(test))]
pub fn get_db_path() -> PathBuf {
    use directories::ProjectDirs;

    if let Ok(path) = std::env::var("ELKIOSKON_DB_PATH") {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }

    if let Some(proj_dirs) = ProjectDirs::from("", "", "com.elkioskon.app") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).ok();
        data_dir.join("el-kioskon-app.db")
    } else {
        PathBuf::from("el-kioskon-app.db")
    }
}
