-- Add migration script here
CREATE TABLE contacts (
    contact_id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    latitude DOUBLE NOT NULL,
    longitude DOUBLE NOT NULL,
    contact_type_id INT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (contact_type_id) REFERENCES contact_types(contact_type_id)
);

CREATE INDEX idx_contacts_contact_type_id ON contacts(contact_type_id);

-- Development
INSERT INTO contacts (name, latitude, longitude, contact_type_id) VALUES
    ('Indomaret Sukamahi', -6.362989058525321, 107.18121119874932, 1),
    ('Sakura Hotel', -6.3681875213804595, 107.17894547792268, 2),
    ('McDonald Deltamas', -6.365373101217244, 107.17883897102675, 1);
