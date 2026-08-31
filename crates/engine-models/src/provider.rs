use crate::error::ModelsError;
use crate::metadata::ModelMetadata;

#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    fn base_url(&self) -> &str;
    fn api_key(&self) -> &str;
    async fn list_models(&self) -> Result<Vec<ModelMetadata>, ModelsError>;
    /// Валидация id по каталогу. Если каталог недоступен — Ok(())
    /// с warn в лог: недоступность /models не должна блокировать чат.
    async fn validate_model(&self, id: &str) -> Result<(), ModelsError>;
}
