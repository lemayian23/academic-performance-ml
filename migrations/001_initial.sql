-- Add migration script here
CREATE TABLE IF NOT EXISTS student_predictions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    study_hours REAL NOT NULL,
    attendance REAL NOT NULL,
    predicted_pass BOOLEAN NOT NULL,
    confidence REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS student_trends (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_name TEXT NOT NULL,
    week INTEGER NOT NULL,
    study_hours REAL NOT NULL,
    attendance REAL NOT NULL,
    predicted_pass BOOLEAN NOT NULL,
    confidence REAL NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS model_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL,
    accuracy REAL NOT NULL,
    features_used TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);