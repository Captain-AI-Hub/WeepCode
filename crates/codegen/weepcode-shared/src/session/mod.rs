use std::path::PathBuf;

pub mod info;

pub use info::Info;

pub fn session_dir(info: &Info) -> PathBuf {
    weepcode_tools::util::weepcode_home::sessions_cwd_dir(&info.cwd).join(info.id.to_string())
}
