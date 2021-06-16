mod entity;
mod po;
pub mod repository;
pub mod route;
mod service;

#[macro_export]
macro_rules! env_var {
    ($key:tt) => {
        std::env::var($key).expect(&format!("missing required environment variable: {}", $key));
    };
}

#[macro_export]
macro_rules! env_u64 {
    ($key:tt) => {
        std::env::var($key)
            .expect(&format!("missing required environment variable: {}", $key))
            .parse::<u64>()
            .expect("an integer is required");
    };
}
