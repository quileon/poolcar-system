-- Add migration script here
CREATE TABLE cars (
    car_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    police_number VARCHAR(255) UNIQUE NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    car_type_id INTEGER NOT NULL REFERENCES car_types(car_type_id),
    tracker_id INTEGER NULL UNIQUE REFERENCES trackers(tracker_id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP NULL
);

CREATE INDEX idx_cars_car_type_id ON cars(car_type_id);
CREATE INDEX idx_cars_tracker_id ON cars(tracker_id);
CREATE INDEX idx_cars_police_number ON cars(police_number);
