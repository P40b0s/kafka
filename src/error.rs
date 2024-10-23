use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error
{   
    #[error("`{0}`")]
    Io(#[from] std::io::Error),
    #[error("`{0}`")]
    SerdeError(#[from] serde_json::Error),
}

impl Serialize for Error
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
    S: serde::Serializer 
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
