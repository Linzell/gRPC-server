// http/user/mod.rs
//
// Copyright Charlie Cohen <linzellart@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.gnu.org/licenses/gpl-3.0.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use axum::{
    routing::{delete, get, post},
    Router,
};
use kiro_database::db_bridge::Database;

mod delete_user;
mod disable_user;
mod read_user;
#[cfg(feature = "mailer")]
mod send_email_to_change_email;
#[cfg(feature = "mailer")]
mod send_email_to_change_password;
mod update_email;
mod update_language;
mod update_notifications;
mod update_password;
mod update_privacy;
mod update_security;
mod update_theme;
#[cfg(feature = "storage")]
mod upload_avatar;

use crate::ClientService;

pub fn user_routes(db: Database) -> Router {
    let service = ClientService::new(db);

    let mut router = Router::new()
        .route("/delete_user", delete(delete_user::delete_user))
        .route("/disable_user", post(disable_user::disable_user))
        .route("/read_user", get(read_user::read_user))
        .route("/update_email", post(update_email::update_email))
        .route("/update_language", post(update_language::update_language))
        .route(
            "/update_notifications",
            post(update_notifications::update_notifications),
        )
        .route("/update_password", post(update_password::update_password))
        .route("/update_privacy", post(update_privacy::update_privacy))
        .route("/update_security", post(update_security::update_security))
        .route("/update_theme", post(update_theme::update_theme));

    #[cfg(feature = "mailer")]
    {
        router = router
            .route(
                "/send_email_to_change_email",
                post(send_email_to_change_email::send_email_to_change_email),
            )
            .route(
                "/send_email_to_change_password",
                post(send_email_to_change_password::send_email_to_change_password),
            );
    }

    #[cfg(feature = "storage")]
    {
        router = router.route("/upload_avatar", post(upload_avatar::upload_avatar));
    }

    router.with_state(service)
}
