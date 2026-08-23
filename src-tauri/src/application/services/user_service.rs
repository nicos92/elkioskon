use bcrypt::{hash, verify};
use chrono::Utc;
use std::sync::Arc;

use crate::domain::entities::{Permission, User, UserPermission};
use crate::domain::repositories::UserRepository;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteUserRepository;

const BCRYPT_COST: u32 = 10;

pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

impl UserService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteUserRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub fn create_user(&self, username: String, password: String) -> Result<User, AppError> {
        let existing = self.repository.find_by_username(&username)?;
        if existing.is_some() {
            return Err(AppError::UsernameExists);
        }

        let hashed_password =
            hash(&password, BCRYPT_COST).map_err(|e| AppError::Hashing(e.to_string()))?;

        let user = User::new(username, hashed_password);
        self.repository.create(&user)
    }

    pub fn login(&self, username: String, password: String) -> Result<User, AppError> {
        let user = self
            .repository
            .find_by_username(&username)?
            .ok_or(AppError::InvalidCredentials)?;

        if !user.active {
            return Err(AppError::UserInactive);
        }

        let is_valid =
            verify(&password, &user.password).map_err(|e| AppError::Hashing(e.to_string()))?;

        if !is_valid {
            return Err(AppError::InvalidCredentials);
        }

        Ok(user)
    }

    pub fn get_user(&self, id: i64) -> Result<User, AppError> {
        self.repository
            .find_by_id(id)?
            .ok_or(AppError::UserNotFound)
    }

    pub fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        self.repository.find_all()
    }

    pub fn update_user(&self, id: i64, username: String, active: bool) -> Result<User, AppError> {
        let mut user = self
            .repository
            .find_by_id(id)?
            .ok_or(AppError::UserNotFound)?;

        let existing = self.repository.find_by_username(&username)?;
        if existing.is_some() && existing.unwrap().id != id {
            return Err(AppError::UsernameExists);
        }

        user.username = username;
        user.active = active;

        self.repository.update(&user)
    }

    pub fn change_password(
        &self,
        actor_id: i64,
        target_user_id: i64,
        current_password: Option<String>,
        new_password: String,
    ) -> Result<User, AppError> {
        if new_password.trim().is_empty() {
            return Err(AppError::EmptyPassword);
        }

        let target = self
            .repository
            .find_by_id(target_user_id)?
            .ok_or(AppError::UserNotFound)?;

        if actor_id == target_user_id {
            let current = current_password.ok_or(AppError::InvalidCredentials)?;
            let is_valid =
                verify(&current, &target.password).map_err(|e| AppError::Hashing(e.to_string()))?;
            if !is_valid {
                return Err(AppError::InvalidCredentials);
            }
        }

        let hashed_password =
            hash(&new_password, BCRYPT_COST).map_err(|e| AppError::Hashing(e.to_string()))?;

        let mut updated = target.clone();
        updated.password = hashed_password;
        updated.modified_at = Utc::now();

        self.repository.update(&updated)
    }

    pub fn delete_user(&self, actor_id: i64, id: i64) -> Result<(), AppError> {
        if actor_id == id {
            return Err(AppError::CannotDeleteSelf);
        }

        let user = self
            .repository
            .find_by_id(id)?
            .ok_or(AppError::UserNotFound)?;

        if user.username == "admin" {
            return Err(AppError::CannotDeleteAdmin);
        }

        self.repository.delete(id)
    }

    pub fn add_permission_to_user(&self, user_id: i64, permission_id: i64) -> Result<(), AppError> {
        let _user = self
            .repository
            .find_by_id(user_id)?
            .ok_or(AppError::UserNotFound)?;

        let permissions = self.repository.get_all_permissions()?;
        if !permissions.iter().any(|p| p.id == permission_id) {
            return Err(AppError::PermissionNotFound);
        }

        self.repository.add_permission(user_id, permission_id)
    }

    pub fn remove_permission_from_user(
        &self,
        user_id: i64,
        permission_id: i64,
    ) -> Result<(), AppError> {
        self.repository.remove_permission(user_id, permission_id)
    }

    pub fn get_user_permissions(&self, user_id: i64) -> Result<Vec<UserPermission>, AppError> {
        self.repository.get_user_permissions(user_id)
    }

    pub fn get_all_permissions(&self) -> Result<Vec<Permission>, AppError> {
        self.repository.get_all_permissions()
    }

    pub fn create_permission(&self, permission_name: String) -> Result<Permission, AppError> {
        let permissions = self.repository.get_all_permissions()?;
        if permissions.iter().any(|p| p.permission == permission_name) {
            return Err(AppError::PermissionExists);
        }

        let permission = Permission::new(permission_name);
        self.repository.create_permission(&permission)
    }

    pub fn has_permission(&self, user_id: i64, permission_name: &str) -> Result<bool, AppError> {
        self.repository.has_permission(user_id, permission_name)
    }

    pub fn get_user_permissions_by_names(&self, user_id: i64) -> Result<Vec<String>, AppError> {
        self.repository.get_user_permissions_by_names(user_id)
    }
}
