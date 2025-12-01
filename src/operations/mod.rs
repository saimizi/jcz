pub mod collection;
pub mod compound;
pub mod compress;
pub mod decompress;
pub mod decrypt;
pub mod encrypt;

#[allow(unused_imports)]
pub use collection::collect_and_compress;
#[allow(unused_imports)]
pub use compound::{compress_compound, compress_compound_batch};
#[allow(unused_imports)]
pub use compress::{compress_file, compress_files};
#[allow(unused_imports)]
pub use decompress::{decompress_file, decompress_files};
#[allow(unused_imports)]
pub use decrypt::{decrypt_file, decrypt_files};
#[allow(unused_imports)]
pub use encrypt::{encrypt_file, encrypt_files};
