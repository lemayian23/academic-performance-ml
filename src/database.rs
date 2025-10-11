use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::{Sqlite, Pool, Row};
use std::error::Error;
use chrono::{DateTime, Utc};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentRecord {
    pub id: i64,
    pub name: String,
    pub study_hours: f64,
    pub attendance: f64,
    pub predicted_pass: bool,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentTrend {
    pub id: i64,
    pub student_name: String,
    pub week: i32,
    pub study_hours: f64,
    pub attendance: f64,
    pub predicted_pass: bool,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelVersion {
    pub id: i64,
    pub version: String,
    pub accuracy: f64,
    pub features_used: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassStatistics {
    pub total_students: i64,
    pub pass_rate: f64,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyTrend {
    pub week: String,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
    pub pass_rate: f64,
    pub prediction_count: i64,
}

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url = "sqlite:student_data.db";
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    pub async fn save_prediction(&self, record: &StudentRecord) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO student_predictions (name, study_hours, attendance, predicted_pass, confidence, created_at)
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

        Ok(result.last_insert_rowid())
    }

    pub async fn get_all_predictions(&self) -> Result<Vec<StudentRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, study_hours, attendance, predicted_pass, confidence, created_at
            FROM student_predictions
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut records = Vec::new();
        for row in rows {
            records.push(StudentRecord {
                id: row.get("id"),
                name: row.get("name"),
                study_hours: row.get("study_hours"),
                attendance: row.get("attendance"),
                predicted_pass: row.get("predicted_pass"),
                confidence: row.get("confidence"),
                created_at: row.get("created_at"),
            });
        }

        Ok(records)
    }

    pub async fn get_recent_predictions(&self, limit: i32) -> Result<Vec<StudentRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, study_hours, attendance, predicted_pass, confidence, created_at
            FROM student_predictions
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut records = Vec::new();
        for row in rows {
            records.push(StudentRecord {
                id: row.get("id"),
                name: row.get("name"),
                study_hours: row.get("study_hours"),
                attendance: row.get("attendance"),
                predicted_pass: row.get("predicted_pass"),
                confidence: row.get("confidence"),
                created_at: row.get("created_at"),
            });
        }

        Ok(records)
    }

    pub async fn save_student_trend(&self, trend: &StudentTrend) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO student_trends (student_name, week, study_hours, attendance, predicted_pass, confidence, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&trend.student_name)
        .bind(trend.week)
        .bind(trend.study_hours)
        .bind(trend.attendance)
        .bind(trend.predicted_pass)
        .bind(trend.confidence)
        .bind(trend.created_at)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_student_trends(&self, student_name: &str) -> Result<Vec<StudentTrend>> {
        let rows = sqlx::query(
            r#"
            SELECT id, student_name, week, study_hours, attendance, predicted_pass, confidence, created_at
            FROM student_trends
            WHERE student_name = ?
            ORDER BY week ASC
            "#
        )
        .bind(student_name)
        .fetch_all(&self.pool)
        .await?;

        let mut trends = Vec::new();
        for row in rows {
            trends.push(StudentTrend {
                id: row.get("id"),
                student_name: row.get("student_name"),
                week: row.get("week"),
                study_hours: row.get("study_hours"),
                attendance: row.get("attendance"),
                predicted_pass: row.get("predicted_pass"),
                confidence: row.get("confidence"),
                created_at: row.get("created_at"),
            });
        }

        Ok(trends)
    }

    pub async fn save_model_version(&self, version: &ModelVersion) -> Result<i64> {
        let result = sqlx::query(
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

        Ok(result.last_insert_rowid())
    }

    pub async fn get_latest_model_version(&self) -> Result<Option<ModelVersion>> {
        let row = sqlx::query(
            r#"
            SELECT id, version, accuracy, features_used, created_at
            FROM model_versions
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ModelVersion {
                id: row.get("id"),
                version: row.get("version"),
                accuracy: row.get("accuracy"),
                features_used: row.get("features_used"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_class_statistics(&self) -> Result<ClassStatistics> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_students,
                AVG(CASE WHEN predicted_pass = 1 THEN 1.0 ELSE 0.0 END) as pass_rate,
                AVG(study_hours) as avg_study_hours,
                AVG(attendance) as avg_attendance
            FROM student_predictions
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ClassStatistics {
            total_students: row.get("total_students"),
            pass_rate: row.get("pass_rate"),
            avg_study_hours: row.get("avg_study_hours"),
            avg_attendance: row.get("avg_attendance"),
        })
    }

    pub async fn get_weekly_trends(&self) -> Result<Vec<WeeklyTrend>> {
        let rows = sqlx::query(
            r#"
            SELECT
                strftime('%Y-%W', created_at) as week,
                AVG(study_hours) as avg_study_hours,
                AVG(attendance) as avg_attendance,
                AVG(CASE WHEN predicted_pass = 1 THEN 1.0 ELSE 0.0 END) as pass_rate,
                COUNT(*) as prediction_count
            FROM student_predictions
            GROUP BY week
            ORDER BY week DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut trends = Vec::new();
        for row in rows {
            trends.push(WeeklyTrend {
                week: row.get("week"),
                avg_study_hours: row.get("avg_study_hours"),
                avg_attendance: row.get("avg_attendance"),
                pass_rate: row.get("pass_rate"),
                prediction_count: row.get("prediction_count"),
            });
        }

        Ok(trends)
    }
}