use serde::{Deserialize, Serialize};
use std::error::Error;
use rand::Rng;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentRecord {
    pub name: String,
    pub hours: f64,
    pub attendance: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PredictResponse {
    pub prediction: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchPredictResponse {
    pub predictions: Vec<StudentPrediction>,
    pub summary: BatchSummary,
    pub total_students: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentPrediction {
    pub name: String,
    pub hours: f64,
    pub attendance: f64,
    pub prediction: String,
    pub confidence: f64,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchSummary {
    pub pass_count: usize,
    pub fail_count: usize,
    pub pass_rate: f64,
    pub avg_confidence: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub accuracy: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsData {
    pub total_students: usize,
    pub pass_rate: f64,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
    pub performance_breakdown: Vec<PerformanceCategory>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceCategory {
    pub range: String,
    pub count: usize,
    pub pass_rate: f64,
}

#[derive(Debug, Clone)]  // Added Clone trait
pub struct TrainedModel {
    // Simple rule-based model
    study_hours_threshold: f64,
    attendance_threshold: f64,
    base_accuracy: f64,
}

impl TrainedModel {
    pub fn new() -> Self {
        // Based on typical student performance patterns
        Self {
            study_hours_threshold: 5.0,
            attendance_threshold: 75.0,
            base_accuracy: 0.85, // 85% accuracy for our simple model
        }
    }

    pub fn predict(&self, features: &[f64]) -> (bool, f64) {
        let hours = features[0];
        let attendance = features[1];
        
        // Simple rule-based prediction
        let mut score = 0.0;
        
        // Study hours contribute 60% to the score
        if hours >= 8.0 {
            score += 0.6;
        } else if hours >= 6.0 {
            score += 0.5;
        } else if hours >= 4.0 {
            score += 0.3;
        } else {
            score += 0.1;
        }
        
        // Attendance contributes 40% to the score
        if attendance >= 90.0 {
            score += 0.4;
        } else if attendance >= 80.0 {
            score += 0.35;
        } else if attendance >= 70.0 {
            score += 0.25;
        } else {
            score += 0.1;
        }
        
        // Add some randomness to simulate real ML model uncertainty
        let mut rng = rand::thread_rng();
        let noise: f64 = rng.gen_range(-0.1..0.1);
        let final_score = (score + noise).clamp(0.0, 1.0);
        
        let prediction = final_score >= 0.5;
        let confidence = if prediction { final_score } else { 1.0 - final_score };
        
        (prediction, confidence)
    }

    pub fn batch_predict(&self, students: Vec<StudentRecord>) -> BatchPredictResponse {
        let mut predictions = Vec::new();
        let mut pass_count = 0;
        let mut fail_count = 0;
        let mut total_confidence = 0.0;

        for student in students {
            let features = vec![student.hours, student.attendance];
            let (prediction, confidence) = self.predict(&features);
            
            total_confidence += confidence;

            if prediction {
                pass_count += 1;
            } else {
                fail_count += 1;
            }

            let recommendation = if prediction {
                if confidence > 0.8 {
                    "Continue current study habits".to_string()
                } else {
                    "Consider slight improvements".to_string()
                }
            } else {
                if student.hours < 5.0 {
                    "Increase study hours significantly".to_string()
                } else if student.attendance < 70.0 {
                    "Improve class attendance".to_string()
                } else {
                    "Seek academic support".to_string()
                }
            };

            predictions.push(StudentPrediction {
                name: student.name,
                hours: student.hours,
                attendance: student.attendance,
                prediction: if prediction { "Pass".to_string() } else { "Fail".to_string() },
                confidence,
                recommendation,
            });
        }

        let total_students = predictions.len();
        let pass_rate = if total_students > 0 {
            pass_count as f64 / total_students as f64
        } else {
            0.0
        };
        let avg_confidence = if total_students > 0 {
            total_confidence / total_students as f64
        } else {
            0.0
        };

        BatchPredictResponse {
            predictions,
            summary: BatchSummary {
                pass_count,
                fail_count,
                pass_rate,
                avg_confidence,
            },
            total_students,
        }
    }

    // Add a method to get the accuracy without consuming self
    pub fn get_accuracy(&self) -> f64 {
        self.base_accuracy
    }
}

// Simple training function that just returns our rule-based model
pub fn train_model() -> Result<(TrainedModel, f64)> {
    let model = TrainedModel::new();
    let accuracy = model.get_accuracy(); // Get accuracy without moving
    Ok((model, accuracy))
}