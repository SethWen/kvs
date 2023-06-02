use super::KvsEngine;

pub struct SledKvsEngine;

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> crate::Result<()> {
        todo!()
    }

    fn get(&mut self, key: String) -> crate::Result<Option<String>> {
        todo!()
    }

    fn remove(&mut self, key: String) -> crate::Result<()> {
        todo!()
    }
}
