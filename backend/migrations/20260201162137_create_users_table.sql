-- Add migration script here
CREATE TABLE users (
    user_id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    user_role_id INT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (user_role_id) REFERENCES user_roles(user_role_id)
);

CREATE INDEX idx_users_user_role_id ON users(user_role_id);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_full_name ON users(full_name);

-- Development
INSERT INTO users (username, email, password, full_name, user_role_id) VALUES
    ('admin', 'quilavr@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$0ydYYMdg6UzgEUDpeDFIDw$XRH3+CyVzn+nhCH/lErA4d/E7nnsY3UODU8KlvFHuUk', 'Quilavr', 1);
