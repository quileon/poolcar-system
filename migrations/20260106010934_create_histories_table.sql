-- Add migration script here
CREATE TABLE histories (
    history_id SERIAL PRIMARY KEY,
    car_id INTEGER NOT NULL REFERENCES cars(car_id),
    activity_id INTEGER NOT NULL REFERENCES activities(activity_id),
    tracker_id INTEGER NOT NULL REFERENCES trackers(tracker_id),
    finished_at TIMESTAMP NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    finished_latitude DECIMAL(10, 8) NOT NULL,
    finished_longitude DECIMAL(11, 8) NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP NULL
);

CREATE INDEX idx_histories_car_id ON histories(car_id);
CREATE INDEX idx_histories_activity_id ON histories(activity_id);
CREATE INDEX idx_histories_tracker_id ON histories(tracker_id);
