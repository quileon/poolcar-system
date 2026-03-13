-- Add migration script here
CREATE TABLE cars (
    car_id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    police_number VARCHAR(255) UNIQUE NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    car_type_id INT NOT NULL,
    tracker_id INT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL,
    FOREIGN KEY (car_type_id) REFERENCES car_types(car_type_id),
    FOREIGN KEY (tracker_id) REFERENCES trackers(tracker_id)
);

CREATE INDEX idx_cars_car_type_id ON cars(car_type_id);
CREATE INDEX idx_cars_tracker_id ON cars(tracker_id);
CREATE INDEX idx_cars_police_number ON cars(police_number);
