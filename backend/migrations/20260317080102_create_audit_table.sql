CREATE TABLE audit (
    audit_id BIGINT AUTO_INCREMENT PRIMARY KEY,
    car_id INT NULL,
    tracker_id INT NOT NULL,
    latitude DOUBLE NOT NULL,
    longitude DOUBLE NOT NULL,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (car_id) REFERENCES cars(car_id),
    FOREIGN KEY (tracker_id) REFERENCES trackers(tracker_id)
);

CREATE INDEX idx_audit_car_id ON audit(car_id);
CREATE INDEX idx_audit_tracker_id ON audit(tracker_id);
CREATE INDEX idx_audit_recorded_at ON audit(recorded_at);
