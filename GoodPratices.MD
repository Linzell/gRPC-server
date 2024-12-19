# List of Best Practices for the Backend Project 📚

*Note: These practices may evolve based on team feedback. Please feel free to share your suggestions.*

## Introduction

This document outlines the best practices for developing the Backend API. These practices ensure code quality, maintainability, and consistency across the project.

### Why These Practices Matter

- **Code Quality**: Ensures reliable and maintainable code
- **Team Collaboration**: Facilitates better team coordination
- **Project Scalability**: Makes the project easier to scale
- **Knowledge Sharing**: Helps new team members understand the codebase

## Project Structure 🏗️

### Workspace Organization

The project uses a Rust workspace to manage multiple related crates. This approach provides several benefits:

- **Modular Development**: Each crate has a specific responsibility
- **Dependency Management**: Shared dependencies are managed at the workspace level
- **Build Optimization**: Faster compilation through shared build artifacts

```toml
[workspace]
resolver = "2"
members = [
    "api/rust",     # Protocol definitions
    "auth",         # Authentication
    "backend",      # Core services
    "client",       # Client SDK
    "database",     # Database
    "mailer",       # Email
    "payment",      # Payments
    "storage",      # Storage
]
```

### Crate Purpose and Responsibilities

#### api
- Defines gRPC protocol contracts
- Handles protocol buffer generation
- Manages API versioning
- Example:
```rust
// api/rust/src/lib.rs
pub mod v1 {
    tonic::include_proto!("v1");
}
```

#### auth
- Manages authentication and authorization
- Handles JWT tokens
- Implements OAuth flows
- Example:
```rust
// auth/src/jwt.rs
pub struct JwtManager {
    secret: String,
    expiration: Duration,
}

impl JwtManager {
    /// Creates a new JWT token for a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user
    /// * `roles` - The roles assigned to the user
    ///
    /// # Returns
    /// A JWT token string or an error
    pub async fn create_token(
        &self,
        user_id: UserId,
        roles: Vec<Role>,
    ) -> Result<String, AuthError> {
        // Implementation
    }
}
```

## Model Mitosis Pattern 🧬

### What is Model Mitosis?

Model Mitosis is a pattern where we maintain different representations of our data models across different layers of the application. Each representation is optimized for its specific use case.

### Why Use Model Mitosis?

1. **Layer Independence**
   - Each layer can evolve independently
   - Changes in one layer don't force changes in others
   - Easier maintenance and refactoring

2. **Type Safety**
   - Compile-time checking of data transformations
   - Clear boundaries between layers
   - Explicit error handling

3. **Performance Optimization**
   - Each model can be optimized for its use case
   - Efficient serialization/deserialization
   - Minimal memory footprint

### Model Layers

#### 1. Protocol Models (api)
```protobuf
// api/rust/proto/user.proto
message User {
    string id = 1;
    string email = 2;
    UserStatus status = 3;
    repeated string roles = 4;
    google.protobuf.Timestamp created_at = 5;
}
```
*Purpose*: Defines the network contract and ensures backward compatibility

#### 2. Domain Models (backend)
```rust
// backend/src/domain/models/user.rs
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,
    status: UserStatus,
    roles: HashSet<Role>,
    created_at: DateTime<Utc>,
}

impl User {
    /// Validates and creates a new user
    pub fn new(
        email: String,
        roles: Vec<String>,
    ) -> Result<Self, DomainError> {
        let email = Email::new(email)?;
        let roles = Role::from_strings(roles)?;

        Ok(Self {
            id: UserId::new(),
            email,
            status: UserStatus::Pending,
            roles: roles.into_iter().collect(),
            created_at: Utc::now(),
        })
    }

    /// Checks if the user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.roles.contains(&Role::Admin)
    }
}
```
*Purpose*: Implements business logic and enforces domain rules

#### 3. Database Models (database)
```rust
// database/src/models/user.rs
#[derive(Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub status: i32,
    pub roles: Vec<String>,
    pub created_at: NaiveDateTime,
}

impl UserRecord {
    /// Converts a database record to a domain model
    pub fn to_domain(self) -> Result<User, DatabaseError> {
        Ok(User {
            id: UserId::from_string(&self.id)?,
            email: Email::new(self.email)?,
            status: UserStatus::from_i32(self.status)?,
            roles: Role::from_strings(self.roles)?,
            created_at: DateTime::from_naive_utc_and_offset(
                self.created_at,
                Utc,
            ),
        })
    }
}
```
*Purpose*: Optimized for database operations and persistence

#### 4. Client Models (client)
```rust
// client/src/models/user.rs
#[derive(Serialize)]
pub struct UserDTO {
    pub id: String,
    pub email: String,
    pub status: String,
}
```

### Model Conversions
```rust
// backend/src/domain/user.rs
impl User {
    pub fn from_proto(proto: api::User) -> Result<Self, Error> {
        Ok(Self {
            id: UserId::new(proto.id)?,
            email: Email::new(proto.email)?,
            status: UserStatus::from_i32(proto.status)?,
        })
    }

    pub fn into_proto(self) -> api::User {
        api::User {
            id: self.id.to_string(),
            email: self.email.to_string(),
            status: self.status as i32,
        }
    }
}
```

## Development Workflow 🛠️

### Setup Requirements
```bash
# Required tools
rustup component add rustfmt clippy rust-src
cargo install cargo-audit cargo-watch cargo-tarpaulin

# Environment setup
cp .env.example .env
docker-compose up -d
```

### Daily Development
```bash
# Start development with watch mode
cargo watch -x fmt -x clippy -x test

# Build and test specific crate
cargo build -p backend
cargo test -p backend

# Generate protocol files
cargo build -p api
```

### Quality Checks
```bash
# Before commit
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo audit
```

## Testing Strategy 🧪

### Test Organization
```rust
// backend/src/services/user.rs
#[cfg(test)]
mod tests {
    use super::*;
    use database::MockDatabase;

    #[tokio::test]
    async fn test_user_creation() {
        let mock_db = MockDatabase::new();
        let service = UserService::new(mock_db);

        let result = service
            .create_user("test@example.com", "password")
            .await;

        assert!(result.is_ok());
    }
}
```

### Integration Testing
```rust
// backend/tests/api_integration.rs
use client::Client;

#[tokio::test]
async fn test_full_user_flow() {
    let client = Client::new("http://localhost:50051");

    // Test user creation
    let user = client
        .create_user("test@example.com", "password")
        .await?;

    // Test authentication
    let token = client
        .authenticate("test@example.com", "password")
        .await?;

    assert!(token.is_valid());
}
```

## Documentation 📚

### Documentation Composition

- Documentation is written in Markdown
- Documentation is written in English
- Documentation follows the GitHub Markdown format (https://guides.github.com/features/mastering-markdown/)

### Code Documentation Composition

- Code documentation follows the Rust documentation format (https://doc.rust-lang.org/reference/comments.html)
- Code documentation is written in English
- **Code documentation is mandatory before submitting a Pull Request**
- Each public function should have a documentation comment

## Code Conventions 📝

### Variable Naming

- Variable names are written in snake_case
- Variable names are explicit
- Variable names are in English
- Use meaningful and descriptive names for variables

### Function Naming

- Function names are written in snake_case
- Function names are explicit
- Function names are in English
- Function names should start with a verb (e.g., get_user, update_profile)

## Commit and Pull Request Conventions 🔄

- Commit messages are written in English
- Commit messages follow the format: `type(scope): subject`

- Pull requests are written in English
- Pull requests follow the format: `type(scope): subject`
- A pull request should not contain more than 500 lines of code
- A pull request must include the progress made and any blocking points
- A pull request should include screenshots of the developed feature (if applicable)
- All tests must pass before a pull request can be merged

### Commit Types and Pull Request Types

- **feat**: Addition of a new feature ✨
- **fix**: Bug fix 🐛
- **docs**: Documentation modification 📝
- **refactor**: Code modification without adding functionality or fixing a bug 🔧
- **perf**: Code modification to improve performance ⚡
- **test**: Addition or modification of tests 🧪
- **build**: Build configuration modification 🏗️
- **ci**: CI configuration modification 🔁
- **chore**: Application configuration modification 🔧
- **revert**: Revert of a previous commit ⏪

### Commit Scopes and Pull Request Scopes

- **database**: Database configuration modification 💾
- **models**: Data model modification 📊
- **services**: Service modification 🛠️
- **utils**: Utility modification 🔧
- **tests**: Test modification 🧪
- **docs**: Documentation modification 📚
- **config**: Application configuration modification ⚙️
- **build**: Build configuration modification 🏗️
- **ci**: CI configuration modification 🔁
- **deps**: Dependency modification 📦
- **other**: Other modification 🔄
