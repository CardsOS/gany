mod error;
mod package;
mod repository;

lazy_static! {
    static ref ARCH: String = std::env::consts::ARCH.to_string();
}
