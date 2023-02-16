use crate::PasteRepository;

use actix_web::web::Bytes;
use once_cell::sync::Lazy;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub struct PasteDiskRepository {}

static PASTE_DIR: Lazy<String> = Lazy::new(|| argh::from_env::<crate::BinArgs>().paste_dir);

impl PasteRepository for PasteDiskRepository {
    fn create(&self, id: &str, content: Bytes) -> Result<(), std::io::Error> {
        let path = Path::new(&*PASTE_DIR);
        let mut file = fs::File::create(path.join(id))?;
        file.write_all(&content)?;
        file.flush()?;
        Ok(())
    }

    fn read(&self, id: &str) -> Option<Bytes> {
        let path = Path::new(&*PASTE_DIR);
        match fs::File::open(path.join(id)) {
            Ok(mut file) => {
                let mut contents = Vec::new();
                match file.read_to_end(&mut contents) {
                    Ok(_) => Some(Bytes::from(contents)),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    fn exists(&self, id: &str) -> bool {
        let path = Path::new(&*PASTE_DIR);
        path.join(id).exists()
    }
}
