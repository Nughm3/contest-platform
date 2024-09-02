use ahash::AHashMap;
use contest::Contest;
use once_cell::sync::OnceCell;

pub mod contest;
pub mod sandbox;
pub mod submit;

pub static CONTESTS: OnceCell<AHashMap<String, Contest>> = OnceCell::new();
