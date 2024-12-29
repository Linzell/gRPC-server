DEFINE TABLE links SCHEMAFULL;

# Links table
DEFINE FIELD user ON links TYPE record<users>;
DEFINE FIELD link_type ON links TYPE string;
DEFINE FIELD expiry ON links TYPE datetime;
