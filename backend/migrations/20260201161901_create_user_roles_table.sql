-- Add migration script here
CREATE TABLE user_roles (
    user_role_id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP NULL
);

INSERT INTO user_roles (name) VALUES
    ('Admin'),
    ('Security'),
    ('Employee'),
ON CONFLICT (name) DO NOTHING;
