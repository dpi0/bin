use actix_web::web::Bytes;
use linked_hash_map::LinkedHashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rand::{distributions::Uniform, thread_rng, Rng};

pub type PasteStore = RwLock<LinkedHashMap<String, Bytes>>;

static BUFFER_SIZE: Lazy<usize> = Lazy::new(|| argh::from_env::<crate::BinArgs>().buffer_size);

/// Ensures `ENTRIES` is less than the size of `BIN_BUFFER_SIZE`. If it isn't then
/// `ENTRIES.len() - BIN_BUFFER_SIZE` elements will be popped off the front of the map.
///
/// During the purge, `ENTRIES` is locked and the current thread will block.
fn purge_old(entries: &mut LinkedHashMap<String, Bytes>) {
    if entries.len() > *BUFFER_SIZE {
        let to_remove = entries.len() - *BUFFER_SIZE;

        for _ in 0..to_remove {
            entries.pop_front();
        }
    }
}

/// Generates a random id, avoiding confusable characters
pub fn generate_id() -> String {
    let valid_chars = "abcdefghjkmnpqrstuvwxyzABCDEFGHJKMNPQRSTUVWXYZ"; // Avoids i, l, and o
    let chars = valid_chars.chars().collect::<Vec<char>>();
    let range = Uniform::from(0..valid_chars.len());
    thread_rng()
        .sample_iter(range)
        .take(12)
        .map(|x| chars[x])
        .collect()
}

/// Stores a paste under the given id
pub fn store_paste(entries: &PasteStore, id: String, content: Bytes) {
    let mut entries = entries.write();

    purge_old(&mut entries);

    entries.insert(id, content);
}

/// Get a paste by id.
///
/// Returns `None` if the paste doesn't exist.
pub fn get_paste(entries: &PasteStore, id: &str) -> Option<Bytes> {
    // need to box the guard until owning_ref understands Pin is a stable address
    entries.read().get(id).map(Bytes::clone)
}
