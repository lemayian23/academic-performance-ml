use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Pool, Sqlite, Row}; // ADDED: Row import
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentRecord {
    pub id: i64,
    pub name: String,
    pub study_hours: f64,
    pub attendance: f64,
    pub predicted_pass: bool,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub id: i64,
    pub version: String,
    pub accuracy: f64,
    pub features_used: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyTrend {
    pub week: usize,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
    pub pass_rate: f64,
    pub total_students: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassStatistics {
    pub total_students: i64,
    pub pass_rate: f64,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
}

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // For simplicity, using in-memory SQLite database
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS predictions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                study_hours REAL NOT NULL,
                attendance REAL NOT NULL,
                predicted_pass BOOLEAN NOT NULL,
                confidence REAL NOT NULL,
                created_at DATETIME NOT NULL
            )
            "#
        ).execute(&pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS model_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version TEXT NOT NULL,
                accuracy REAL NOT NULL,
                features_used TEXT NOT NULL,
                created_at DATETIME NOT NULL
            )
            "#
        ).execute(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn save_prediction(&self, record: &StudentRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO predictions (name, study_hours, attendance, predicted_pass, confidence, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&record.name)
        .bind(record.study_hours)
        .bind(record.attendance)
        .bind(record.predicted_pass)
        .bind(record.confidence)
        .bind(record.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_model_version(&self, version: &ModelVersion) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO model_versions (version, accuracy, features_used, created_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(&version.version)
        .bind(version.accuracy)
        .bind(&version.features_used)
        .bind(version.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_predictions(&self) -> Result<Vec<StudentRecord>, sqlx::Error> {
        // Use query instead of query_as! to avoid macro issues
        let rows = sqlx::query(
            r#"
            SELECT id, name, study_hours, attendance, predicted_pass, confidence, created_at
            FROM predictions
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut predictions = Vec::new();
        for row in rows {
            predictions.push(StudentRecord {
                id: row.get("id"),
                name: row.get("name"),
                study_hours: row.get("study_hours"),
                attendance: row.get("attendance"),
                predicted_pass: row.get("predicted_pass"),
                confidence: row.get("confidence"),
                created_at: row.get("created_at"),
            });
        }

        Ok(predictions)
    }

    pub async fn get_class_statistics(&self) -> Result<ClassStatistics, sqlx::Error> {
        // Use query instead of query! to avoid macro issues
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_students,
                AVG(study_hours) as avg_study_hours,
                AVG(attendance) as avg_attendance,
                AVG(CASE WHEN predicted_pass THEN 1.0 ELSE 0.0 END) as pass_rate
            FROM predictions
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ClassStatistics {
            total_students: row.get::<i64, _>("total_students"),
            avg_study_hours: row.get::<Option<f64>, _>("avg_study_hours").unwrap_or(0.0),
            avg_attendance: row.get::<Option<f64>, _>("avg_attendance").unwrap_or(0.0),
            pass_rate: row.get::<Option<f64>, _>("pass_rate").unwrap_or(0.0),
        })
    }

    pub async fn get_weekly_trends(&self) -> Result<Vec<WeeklyTrend>, sqlx::Error> {
        // For demo purposes, return mock data
        // In a real implementation, you'd group by week and calculate averages
        let mock_trends = vec![
            WeeklyTrend {
                week: 1,
                avg_study_hours: 5.2,
                avg_attendance: 75.0,
                pass_rate: 0.65,
                total_students: 45,
            },
            WeeklyTrend {
                week: 2,
                avg_study_hours: 5.8,
                avg_attendance: 78.0,
                pass_rate: 0.72,
                total_students: 52,
            },
            WeeklyTrend {
                week: 3,
                avg_study_hours: 6.1,
                avg_attendance: 80.0,
                pass_rate: 0.78,
                total_students: 48,
            },
            WeeklyTrend {
                week: 4,
                avg_study_hours: 6.5,
                avg_attendance: 82.0,
                pass_rate: 0.81,
                total_students: 55,
            },
        ];

        Ok(mock_trends)
    }
}