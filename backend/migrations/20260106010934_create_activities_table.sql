-- Add migration script here
CREATE TABLE activities (
    activity_id SERIAL PRIMARY KEY,
    car_id INTEGER NULL REFERENCES cars(car_id),
    contact_id INTEGER NOT NULL REFERENCES contacts(contact_id),
    activity_type_id INTEGER NOT NULL REFERENCES activity_types(activity_type_id),
    tracker_id INTEGER NULL REFERENCES trackers(tracker_id),
    started_at TIMESTAMP NULL DEFAULT NOW(),
    finished_at TIMESTAMP NULL,
    finished_latitude DECIMAL(10, 8) NULL,
    finished_longitude DECIMAL(11, 8) NULL,
    description TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP NULL
);

CREATE INDEX idx_activities_car_id ON activities(car_id);
CREATE INDEX idx_activities_contact_id ON activities(contact_id);
CREATE INDEX idx_activities_activity_type_id ON activities(activity_type_id);
CREATE INDEX idx_activities_tracker_id ON activities(tracker_id);
CREATE INDEX idx_activities_finished_at ON activities(finished_at);
