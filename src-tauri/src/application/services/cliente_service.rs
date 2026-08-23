use std::sync::Arc;

use crate::domain::entities::Cliente;
use crate::domain::repositories::ClienteRepository;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteClienteRepository;

pub struct ClienteService {
    repository: Arc<dyn ClienteRepository>,
}

impl Default for ClienteService {
    fn default() -> Self {
        Self::new()
    }
}

impl ClienteService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteClienteRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn ClienteRepository>) -> Self {
        Self { repository }
    }

    pub fn create(
        &self,
        nombre: Option<String>,
        apellido: Option<String>,
        telefono: Option<String>,
        email: Option<String>,
        direccion: Option<String>,
    ) -> Result<Cliente, AppError> {
        let nombre = normalize(nombre);
        let apellido = normalize(apellido);
        let telefono = normalize(telefono);
        let email = normalize(email);
        let direccion = normalize(direccion);

        validate_contact_data(&nombre, &apellido, &telefono, &email, &direccion)?;

        let new_cliente = Cliente::new(nombre, apellido, telefono, email, direccion);
        self.repository.create(&new_cliente)
    }

    pub fn get_all(&self) -> Result<Vec<Cliente>, AppError> {
        self.repository.find_all()
    }

    pub fn get_by_id(&self, id: i64) -> Result<Cliente, AppError> {
        self.repository
            .find_by_id(id)?
            .ok_or(AppError::ClienteNotFound)
    }

    pub fn get_default(&self) -> Result<Cliente, AppError> {
        self.repository
            .find_default()?
            .ok_or(AppError::ClienteDefectoNotFound)
    }

    pub fn update(&self, cliente: &Cliente) -> Result<Cliente, AppError> {
        let mut existing = self
            .repository
            .find_by_id(cliente.id)?
            .ok_or(AppError::ClienteNotFound)?;

        let nombre = normalize(cliente.nombre.clone());
        let apellido = normalize(cliente.apellido.clone());
        let telefono = normalize(cliente.telefono.clone());
        let email = normalize(cliente.email.clone());
        let direccion = normalize(cliente.direccion.clone());

        validate_contact_data(&nombre, &apellido, &telefono, &email, &direccion)?;

        existing.nombre = nombre;
        existing.apellido = apellido;
        existing.telefono = telefono;
        existing.email = email;
        existing.direccion = direccion;
        existing.updated_at = chrono::Utc::now().to_rfc3339();

        self.repository.update(&existing)
    }

    pub fn delete(&self, id: i64) -> Result<(), AppError> {
        let _existing = self
            .repository
            .find_by_id(id)?
            .ok_or(AppError::ClienteNotFound)?;

        if let Some(default) = self.repository.find_default()? {
            if default.id == id {
                return Err(AppError::NoSePuedeEliminarClienteDefecto);
            }
        }

        self.repository.delete(id)
    }
}

fn normalize(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn validate_contact_data(
    nombre: &Option<String>,
    apellido: &Option<String>,
    telefono: &Option<String>,
    email: &Option<String>,
    direccion: &Option<String>,
) -> Result<(), AppError> {
    if nombre.is_none()
        && apellido.is_none()
        && telefono.is_none()
        && email.is_none()
        && direccion.is_none()
    {
        return Err(AppError::ClienteSinDatosDeContacto);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::cliente_repository::MockClienteRepository;
    use mockall::predicate::*;

    fn sample_cliente() -> Cliente {
        Cliente::new(
            Some("Juan".to_string()),
            Some("Pérez".to_string()),
            Some("555-1234".to_string()),
            None,
            None,
        )
    }

    #[test]
    fn create_accepts_single_contact_field() {
        let mut repo = MockClienteRepository::new();
        repo.expect_create().returning(|c| Ok(c.clone()));
        let service = ClienteService::with_repository(Arc::new(repo));

        let cliente = service.create(None, None, Some("555-0000".to_string()), None, None);
        assert!(cliente.is_ok());
        assert_eq!(cliente.unwrap().telefono.as_deref(), Some("555-0000"));
    }

    #[test]
    fn create_normalizes_whitespace_and_empty_strings() {
        let mut repo = MockClienteRepository::new();
        repo.expect_create().returning(|c| Ok(c.clone()));
        let service = ClienteService::with_repository(Arc::new(repo));

        let cliente = service.create(
            Some("   ".to_string()),
            Some("Final".to_string()),
            Some("".to_string()),
            None,
            None,
        );
        let c = cliente.unwrap();
        assert!(c.nombre.is_none());
        assert_eq!(c.apellido.as_deref(), Some("Final"));
        assert!(c.telefono.is_none());
    }

    #[test]
    fn create_rejects_all_empty_fields() {
        let service = ClienteService::new();
        let err = service.create(None, None, None, None, None).unwrap_err();
        assert!(matches!(err, AppError::ClienteSinDatosDeContacto));
    }

    #[test]
    fn create_rejects_all_whitespace_fields() {
        let service = ClienteService::new();
        let err = service
            .create(
                Some(" ".to_string()),
                Some("  ".to_string()),
                Some("\t".to_string()),
                None,
                None,
            )
            .unwrap_err();
        assert!(matches!(err, AppError::ClienteSinDatosDeContacto));
    }

    #[test]
    fn update_keeps_timestamps_and_revalidates() {
        let mut repo = MockClienteRepository::new();
        let existing = sample_cliente();
        repo.expect_find_by_id()
            .with(eq(existing.id))
            .return_once(move |_| Ok(Some(existing)));
        repo.expect_update().returning(|c| Ok(c.clone()));
        let service = ClienteService::with_repository(Arc::new(repo));

        let mut cliente = sample_cliente();
        cliente.email = Some("nuevo@mail.com".to_string());
        let updated = service.update(&cliente).unwrap();
        assert_eq!(updated.email.as_deref(), Some("nuevo@mail.com"));
    }

    #[test]
    fn update_rejects_all_empty_fields() {
        let mut repo = MockClienteRepository::new();
        let existing = sample_cliente();
        repo.expect_find_by_id()
            .with(eq(existing.id))
            .return_once(move |_| Ok(Some(existing)));
        let service = ClienteService::with_repository(Arc::new(repo));

        let mut cliente = sample_cliente();
        cliente.nombre = None;
        cliente.apellido = None;
        cliente.telefono = None;
        cliente.email = Some("   ".to_string());
        cliente.direccion = None;
        let err = service.update(&cliente).unwrap_err();
        assert!(matches!(err, AppError::ClienteSinDatosDeContacto));
    }

    #[test]
    fn update_returns_not_found_for_missing_id() {
        let mut repo = MockClienteRepository::new();
        repo.expect_find_by_id()
            .with(eq(99))
            .return_once(|_| Ok(None));
        let service = ClienteService::with_repository(Arc::new(repo));

        let mut cliente = sample_cliente();
        cliente.id = 99;
        let err = service.update(&cliente).unwrap_err();
        assert!(matches!(err, AppError::ClienteNotFound));
    }

    #[test]
    fn delete_rejects_default_client() {
        let mut repo = MockClienteRepository::new();
        repo.expect_find_by_id()
            .with(eq(1))
            .return_once(|_| Ok(Some(sample_cliente())));
        let mut default = Cliente::new(
            Some("Consumidor".to_string()),
            Some("Final".to_string()),
            None,
            None,
            None,
        );
        default.id = 1;
        repo.expect_find_default().return_once(|| Ok(Some(default)));
        let service = ClienteService::with_repository(Arc::new(repo));

        let err = service.delete(1).unwrap_err();
        assert!(matches!(err, AppError::NoSePuedeEliminarClienteDefecto));
    }

    #[test]
    fn delete_allows_non_default_client() {
        let mut repo = MockClienteRepository::new();
        let mut cliente = sample_cliente();
        cliente.id = 5;
        repo.expect_find_by_id()
            .with(eq(5))
            .return_once(move |_| Ok(Some(cliente)));
        repo.expect_find_default().return_once(|| {
            Ok(Some(Cliente::new(
                Some("Consumidor".to_string()),
                Some("Final".to_string()),
                None,
                None,
                None,
            )))
        });
        repo.expect_delete().with(eq(5)).return_once(|_| Ok(()));
        let service = ClienteService::with_repository(Arc::new(repo));

        assert!(service.delete(5).is_ok());
    }

    #[test]
    fn delete_returns_not_found_for_missing_id() {
        let mut repo = MockClienteRepository::new();
        repo.expect_find_by_id()
            .with(eq(42))
            .return_once(|_| Ok(None));
        let service = ClienteService::with_repository(Arc::new(repo));

        let err = service.delete(42).unwrap_err();
        assert!(matches!(err, AppError::ClienteNotFound));
    }

    #[test]
    fn get_default_propagates_not_found() {
        let mut repo = MockClienteRepository::new();
        repo.expect_find_default().return_once(|| Ok(None));
        let service = ClienteService::with_repository(Arc::new(repo));

        let err = service.get_default().unwrap_err();
        assert!(matches!(err, AppError::ClienteDefectoNotFound));
    }
}
