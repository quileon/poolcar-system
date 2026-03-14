-- Add migration script here
CREATE TABLE activities (
    activity_id INT AUTO_INCREMENT PRIMARY KEY,
    car_id INT NULL,
    contact_id INT NOT NULL,
    activity_type_id INT NOT NULL,
    tracker_id INT NULL,
    started_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at DATETIME NULL,
    finished_latitude DECIMAL(10, 8) NULL,
    finished_longitude DECIMAL(11, 8) NULL,
    description TEXT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (car_id) REFERENCES cars(car_id),
    FOREIGN KEY (contact_id) REFERENCES contacts(contact_id),
    FOREIGN KEY (activity_type_id) REFERENCES activity_types(activity_type_id),
    FOREIGN KEY (tracker_id) REFERENCES trackers(tracker_id)
);

CREATE INDEX idx_activities_car_id ON activities(car_id);
CREATE INDEX idx_activities_contact_id ON activities(contact_id);
CREATE INDEX idx_activities_activity_type_id ON activities(activity_type_id);
CREATE INDEX idx_activities_tracker_id ON activities(tracker_id);
CREATE INDEX idx_activities_finished_at ON activities(finished_at);
