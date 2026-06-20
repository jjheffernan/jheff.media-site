use crate::error::{ServiceError, ServiceResult};
use crate::models::user_token::UserToken;
use actix_web::http::StatusCode;
use bcrypt::{hash, verify, DEFAULT_COST};
use bson::{doc, oid::ObjectId};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const USERS_COLLECTION: &str = "users";
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_session: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDTO {
    pub email: String,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_session: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginDTO {
    pub email_or_username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfoDTO {
    pub email: String,
    pub username: String,
    pub login_session: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserDTO {
    pub email: String,
    pub username: String,
}

impl From<LoginInfoDTO> for PublicUserDTO {
    fn from(li: LoginInfoDTO) -> Self {
        PublicUserDTO {
            email: li.email,
            username: li.username,
        }
    }
}

impl User {
    pub async fn signup(user: User, db: &Database) -> Result<ObjectId, String> {
        if Self::find_by_email_or_username(user.username.as_str(), db)
            .await
            .is_some()
        {
            return Err("USER_ALREADY_EXISTS".to_string());
        }
        if Self::find_by_email_or_username(user.email.as_str(), db)
            .await
            .is_some()
        {
            return Err("USER_ALREADY_EXISTS".to_string());
        }
        let coll = db.collection::<User>(USERS_COLLECTION);
        let hashed_pwd = hash(&user.password, DEFAULT_COST).unwrap();
        let user = Self {
            password: hashed_pwd,
            ..user
        };
        match coll.insert_one(user).await {
            Ok(res) => Ok(res.inserted_id.as_object_id().unwrap().clone()),
            Err(_err) => Err("MONGO_ERROR".to_string()),
        }
    }

    pub async fn login(login: &LoginDTO, db: &Database) -> ServiceResult<LoginInfoDTO> {
        match User::find_by_email_or_username(login.email_or_username.as_str(), db).await {
            Some(user_to_verify) => {
                if !user_to_verify.password.is_empty()
                    && verify(&login.password, &user_to_verify.password).unwrap()
                {
                    let login_session_str = User::generate_login_session();
                    if User::update_login_session(
                        &user_to_verify.id.unwrap(),
                        &login_session_str,
                        db,
                    )
                    .await
                    {
                        return Ok(LoginInfoDTO {
                            email: user_to_verify.email,
                            username: user_to_verify.username,
                            login_session: login_session_str,
                        });
                    }
                }
                Err(ServiceError::new(StatusCode::BAD_REQUEST, "BAD_PASSWORD"))
            }
            None => Err(ServiceError::new(StatusCode::NOT_FOUND, "USER_NOT_FOUND")),
        }
    }

    pub async fn logout(user_id: ObjectId, db: &Database) {
        let coll = db.collection::<User>(USERS_COLLECTION);
        match coll
            .find_one_and_update(
                doc! {"_id": user_id},
                doc! {"$unset": {"loginSession": ""}},
            )
            .await
        {
            Ok(_doc) => (),
            Err(err) => {
                warn!("An error occured while unsetting session: {}", err);
            }
        }
    }

    pub async fn is_valid_login_session(user_token: &UserToken, db: &Database) -> bool {
        let coll = db.collection::<User>(USERS_COLLECTION);
        match coll
            .count_documents(doc! {
                "username": &user_token.user,
                "loginSession": &user_token.login_session
            })
            .await
        {
            Ok(num) => num == 1,
            Err(err) => {
                warn!(
                    "An error occured while checking validity of user session: {}",
                    err
                );
                false
            }
        }
    }

    pub async fn find_by_email_or_username(eou: &str, db: &Database) -> Option<Self> {
        let coll = db.collection::<User>(USERS_COLLECTION);
        match coll
            .find_one(doc! {"$or": [{"username": eou}, {"email": eou}]})
            .await
        {
            Ok(opt_user) => opt_user,
            Err(err) => {
                warn!("An error occured while finding a user by username: {}", err);
                None
            }
        }
    }

    pub fn generate_login_session() -> String {
        Uuid::new_v4().to_string()
    }

    pub async fn update_login_session(
        user_id: &ObjectId,
        login_session: &str,
        db: &Database,
    ) -> bool {
        let coll = db.collection::<User>(USERS_COLLECTION);
        match coll
            .find_one_and_update(
                doc! {"_id": user_id},
                doc! {"$set": {"loginSession": login_session}},
            )
            .await
        {
            Ok(_doc) => true,
            Err(err) => {
                warn!("An error occured while unsetting session: {}", err);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_dto_deserializes_camel_case_json() {
        let dto: LoginDTO =
            serde_json::from_str(r#"{"emailOrUsername":"alice","password":"secret"}"#).unwrap();
        assert_eq!(dto.email_or_username, "alice");
        assert_eq!(dto.password, "secret");
    }

    #[test]
    fn public_user_from_login_info_drops_session() {
        let info = LoginInfoDTO {
            email: "a@b.com".into(),
            username: "bob".into(),
            login_session: "sess".into(),
        };
        let public = PublicUserDTO::from(info);
        assert_eq!(public.email, "a@b.com");
        assert_eq!(public.username, "bob");
    }

    #[test]
    fn generate_login_session_is_non_empty_uuid() {
        let session = User::generate_login_session();
        assert!(!session.is_empty());
        assert!(session.contains('-'));
    }
}
