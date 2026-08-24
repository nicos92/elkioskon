use std::sync::Arc;

use crate::domain::entities::Proveedor;
use crate::domain::repositories::ProveedorRepository;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteProveedorRepository;

pub struct ProveedorService {
    repository: Arc<dyn ProveedorRepository>,
}

impl Default for ProveedorService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProveedorService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteProveedorRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn ProveedorRepository>) -> Self {
        Self { repository }
    }

    pub fn create(
        &self,
        proveedor: String,
        nombre: String,
        cuit: Option<String>,
        tel: Option<String>,
        email: Option<String>,
        observacion: Option<String>,
    ) -> Result<Proveedor, AppError> {
        if let Some(ref c) = cuit {
            if !c.is_empty() {
                let existing = self.repository.find_by_cuit(c)?;
                if existing.is_some() {
                    return Err(AppError::DuplicateCuit);
                }
            }
        }

        let new_proveedor = Proveedor::new(proveedor, nombre, cuit, tel, email, observacion);
        self.repository.create(&new_proveedor)
    }

    pub fn get_all(&self) -> Result<Vec<Proveedor>, AppError> {
        self.repository.find_all()
    }

    pub fn get_by_id(&self, id: i64) -> Result<Proveedor, AppError> {
        self.repository
            .find_by_id(id)?
            .ok_or(AppError::ProveedorNotFound)
    }

    pub fn update(&self, proveedor: &Proveedor) -> Result<Proveedor, AppError> {
        let mut existing = self
            .repository
            .find_by_id(proveedor.id)?
            .ok_or(AppError::ProveedorNotFound)?;

        if existing.is_default() {
            return Err(AppError::NoSePuedeModificarProveedorDefecto);
        }

        if let Some(ref c) = proveedor.cuit {
            if !c.is_empty() {
                let existing_cuit = self.repository.find_by_cuit(c)?;
                if let Some(ref ec) = existing_cuit {
                    if ec.id != proveedor.id {
                        return Err(AppError::DuplicateCuit);
                    }
                }
            }
        }

        existing.proveedor = proveedor.proveedor.clone();
        existing.nombre = proveedor.nombre.clone();
        existing.cuit = proveedor.cuit.clone();
        existing.tel = proveedor.tel.clone();
        existing.email = proveedor.email.clone();
        existing.observacion = proveedor.observacion.clone();

        self.repository.update(&existing)
    }

    pub fn delete(&self, id: i64) -> Result<(), AppError> {
        let existing = self
            .repository
            .find_by_id(id)?
            .ok_or(AppError::ProveedorNotFound)?;

        if existing.is_default() {
            return Err(AppError::NoSePuedeEliminarProveedorDefecto);
        }

        let has_articulos = self.repository.has_articulos(id)?;
        if has_articulos {
            return Err(AppError::ProveedorHasArticulos);
        }

        self.repository.delete(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::DEFAULT_PROVEEDOR_NOMBRE;
    use crate::domain::repositories::proveedor_repository::MockProveedorRepository;
    use mockall::predicate::*;

    fn proveedor_con_id(nombre: &str, id: i64) -> Proveedor {
        let mut p = Proveedor::new(
            nombre.to_string(),
            nombre.to_string(),
            None,
            None,
            None,
            None,
        );
        p.id = id;
        p
    }

    #[test]
    fn update_rejects_default_proveedor() {
        let mut repo = MockProveedorRepository::new();
        repo.expect_find_by_id()
            .with(eq(1))
            .returning(|_| Ok(Some(proveedor_con_id(DEFAULT_PROVEEDOR_NOMBRE, 1))));
        let service = ProveedorService::with_repository(Arc::new(repo));

        let request = proveedor_con_id(DEFAULT_PROVEEDOR_NOMBRE, 1);
        let err = service.update(&request).unwrap_err();
        assert!(
            matches!(err, AppError::NoSePuedeModificarProveedorDefecto),
            "{:?}",
            err
        );
    }

    #[test]
    fn delete_rejects_default_proveedor() {
        let mut repo = MockProveedorRepository::new();
        repo.expect_find_by_id()
            .with(eq(1))
            .returning(|_| Ok(Some(proveedor_con_id(DEFAULT_PROVEEDOR_NOMBRE, 1))));
        let service = ProveedorService::with_repository(Arc::new(repo));

        let err = service.delete(1).unwrap_err();
        assert!(
            matches!(err, AppError::NoSePuedeEliminarProveedorDefecto),
            "{:?}",
            err
        );
    }

    #[test]
    fn delete_allows_regular_proveedor_sin_articulos() {
        let mut repo = MockProveedorRepository::new();
        repo.expect_find_by_id()
            .with(eq(5))
            .returning(|_| Ok(Some(proveedor_con_id("ElectroSur", 5))));
        repo.expect_has_articulos()
            .with(eq(5))
            .returning(|_| Ok(false));
        repo.expect_delete().with(eq(5)).returning(|_| Ok(()));
        let service = ProveedorService::with_repository(Arc::new(repo));

        assert!(service.delete(5).is_ok());
    }

    #[test]
    fn update_allows_regular_proveedor() {
        let mut repo = MockProveedorRepository::new();
        repo.expect_find_by_id()
            .with(eq(5))
            .returning(|_| Ok(Some(proveedor_con_id("ElectroSur", 5))));
        repo.expect_update().returning(|p| Ok(p.clone()));
        let service = ProveedorService::with_repository(Arc::new(repo));

        let mut request = proveedor_con_id("ElectroSur", 5);
        request.nombre = "Nombre nuevo".to_string();
        let updated = service.update(&request).unwrap();
        assert_eq!(updated.nombre, "Nombre nuevo");
    }
}
