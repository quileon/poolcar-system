-- Add migration script here
CREATE TABLE trackers (
    tracker_id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL
);

-- Development
-- INSERT INTO trackers (name) VALUES
--     ('Alpha'),
--     ('Beta'),
--     ('Gamma');
