-- Add explicit Keycloak subject mapping as API identity key.

ALTER TABLE marketplace_users
ADD COLUMN IF NOT EXISTS keycloak_sub TEXT UNIQUE;

