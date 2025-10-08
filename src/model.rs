use ndarray::Array2;
use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct ModelInfo {
    pub accuracy: f64,
}

#[derive(Serialize)]
pub struct PredictResponse {
    pub prediction: String,
    pub confidence: f64,
}

#[derive(Serialize)]
pub struct AnalyticsData {
    pub total_students: usize,
    pub pass_rate: f64,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
    pub performance_breakdown: Vec<PerformanceCategory>,
}

#[derive(Serialize)]
pub struct PerformanceCategory {
    pub range: String,
    pub count: usize,
    pub pass_rate: f64,
}

#[derive(Deserialize)]
pub struct StudentRecord {
    pub name: String,
    pub hours: f64,
    pub attendance: f64,
}

#[derive(Serialize)]
pub struct BatchPrediction {
    pub name: String,
    pub hours: f64,
    pub attendance: f64,
    pub prediction: String,
    pub confidence: f64,
    pub recommendation: String,
}

#[derive(Serialize)]
pub struct BatchResult {
    pub total_students: usize,
    pub predictions: Vec<BatchPrediction>,
    pub summary: BatchSummary,
}

#[derive(Serialize)]
pub struct BatchSummary {
    pub pass_count: usize,
    pub fail_count: usize,
    pub avg_confidence: f64,
    pub pass_rate: f64,
}

pub struct TrainedModel {
    model: linfa_logistic::FittedLogisticRegression<f64, bool>,
}

impl TrainedModel {
    pub fn predict(&self, features: &[f64]) -> (bool, f64) {
        let feature_array = Array2::from_shape_vec((1, 2), features.to_vec()).unwrap();
        let prediction = self.model.predict(&feature_array);
        let probabilities = self.model.predict_probabilities(&feature_array);
        let confidence = if prediction[0] { 
            probabilities[[0]] 
        } else { 
            1.0 - probabilities[[0]] 
        };
        
        (prediction[0], confidence)
    }

    pub fn batch_predict(&self, students: Vec<StudentRecord>) -> BatchResult {
        let mut predictions = Vec::new();
        let mut pass_count = 0;
        let mut total_confidence = 0.0;

        for student in students {
            let (prediction, confidence) = self.predict(&[student.hours, student.attendance]);
            
            if prediction { pass_count += 1; }
            total_confidence += confidence;

            let recommendation = if prediction {
                if student.hours >= 6.0 && student.attendance >= 80.0 {
                    "Maintain current performance".to_string()
                } else {
                    "Good progress, aim for 6+ hours and 80%+ attendance".to_string()
                }
            } else {
                "Needs improvement - increase study time and attendance".to_string()
            };

            predictions.push(BatchPrediction {
                name: student.name,
                hours: student.hours,
                attendance: student.attendance,
                prediction: if prediction { "Pass".to_string() } else { "Fail".to_string() },
                confidence,
                recommendation,
            });
        }

        let total_students = predictions.len();
        BatchResult {
            total_students,
            predictions,
            summary: BatchSummary {
                pass_count,
                fail_count: total_students - pass_count,
                avg_confidence: if total_students > 0 { total_confidence / total_students as f64 } else { 0.0 },
                pass_rate: if total_students > 0 { pass_count as f64 / total_students as f64 } else { 0.0 },
            },
        }
    }
}

pub fn train_model(data: Array2<f64>) -> Result<(TrainedModel, f64), Box<dyn std::error::Error>> {
    use crate::data::{enhance_data_if_needed, calculate_accuracy};
    
    let (features, targets) = enhance_data_if_needed(&data)?;
    
    println!(" Training logistic regression model...");
    let dataset = Dataset::new(features.clone(), targets.clone());
    let model = LogisticRegression::default()
        .max_iterations(100)
        .fit(&dataset)
        .expect("Failed to train model");

    let predictions = model.predict(&features);
    let accuracy = calculate_accuracy(&predictions, &targets);

    Ok((TrainedModel { model }, accuracy))
}