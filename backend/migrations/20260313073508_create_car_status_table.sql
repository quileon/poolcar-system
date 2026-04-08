-- Add migration script here
CREATE TABLE car_status (
    car_status_id INT AUTO_INCREMENT PRIMARY KEY,
    car_id INT NOT NULL,
    gas_level DOUBLE NOT NULL,
    kilometres DOUBLE NOT NULL,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL,
    FOREIGN KEY (car_id) REFERENCES cars(car_id)
);

CREATE INDEX idx_car_status_car_id ON car_status(car_id);
CREATE INDEX idx_car_status_recorded_at ON car_status(recorded_at);
