CREATE TABLE car_audit (
    car_audit_id BIGINT AUTO_INCREMENT PRIMARY KEY,
    car_id INT NOT NULL,
    tracker_id INT NOT NULL,
    latitude DOUBLE NOT NULL,
    longitude DOUBLE NOT NULL,
    recorded_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (car_id) REFERENCES cars(car_id),
    FOREIGN KEY (tracker_id) REFERENCES trackers(tracker_id)
);

CREATE INDEX idx_car_audit_car_id ON car_audit(car_id);
CREATE INDEX idx_car_audit_tracker_id ON car_audit(tracker_id);
CREATE INDEX idx_car_audit_recorded_at ON car_audit(recorded_at);
