DEFINE TABLE sessions SCHEMAFULL;

# Session table
DEFINE FIELD session_key ON sessions TYPE string;
DEFINE FIELD expires_at ON sessions TYPE datetime VALUE time::now() + 7d;
DEFINE FIELD user_id ON sessions TYPE record<users>;
DEFINE FIELD ip_address ON sessions TYPE option<string>;
DEFINE FIELD is_admin ON sessions TYPE bool;
