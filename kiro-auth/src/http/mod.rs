// http/mod.rs
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
    routing::{get, post},
    Router,
};
use kiro_database::db_bridge::Database;

mod login;
// mod logout;
// mod register;

use crate::AuthService;

pub fn auth_routes(db: Database) -> Router {
    let service = AuthService::new(db);

    Router::new()
        .route("/login", post(login::login))
        .route("/logout", get(unimplemented!()))
        .route("/register", post(unimplemented!()))
        .with_state(service)
}
