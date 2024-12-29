// utils/doc.rs
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

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // # Authentication
        kiro_client::login::login,
        kiro_client::logout::logout,
        kiro_client::register::register,
        // # User
    ),
    components(
        schemas(
            // # Authentication
            kiro_api::auth::v1::AuthRequest,
            kiro_api::auth::v1::Session,
            // # User
        )
    ),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "auth", description = "Authentication endpoints"),
    ),
    info(
        title = "Kiro API",
        version = "0.0.2",
        description = "API for Kiro",
    )
)]

pub struct ApiDoc;
