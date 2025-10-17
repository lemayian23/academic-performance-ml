use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub struct StudentTrend {
    pub student_name: String,
    pub trend: String, // "Improving", "Declining", "Stable"
    pub current_performance: String,
    pub recommendation: String,
}

#[derive(Serialize, Clone)]
pub struct ClassTrends {
    pub weeks: Vec<usize>,
    pub avg_study_hours: Vec<f64>,
    pub avg_attendance: Vec<f64>,
    pub pass_rates: Vec<f64>,
}

pub struct TrendsAnalyzer;

impl TrendsAnalyzer {
    pub fn new() -> Self {
        TrendsAnalyzer
    }

    pub fn generate_student_trend(&self, student_name: &str, historical_data: Vec<(f64, f64)>) -> StudentTrend {
        if historical_data.is_empty() {
            return StudentTrend {
                student_name: student_name.to_string(),
                trend: "Insufficient Data".to_string(),
                current_performance: "Unknown".to_string(),
                recommendation: "Need more data points".to_string(),
            };
        }

        let (current_hours, current_attendance) = historical_data.last().unwrap();
        
        let trend = if historical_data.len() >= 2 {
            let (first_hours, first_attendance) = historical_data.first().unwrap();
            let hours_change = current_hours - first_hours;
            let attendance_change = current_attendance - first_attendance;
            
            if hours_change > 1.0 && attendance_change > 5.0 {
                "Improving"
            } else if hours_change < -1.0 || attendance_change < -5.0 {
                "Declining"
            } else {
                "Stable"
            }
        } else {
            "Stable"
        };

        let performance = if *current_hours >= 8.0 && *current_attendance >= 85.0 {
            "Excellent"
        } else if *current_hours >= 6.0 && *current_attendance >= 75.0 {
            "Good"
        } else if *current_hours >= 4.0 && *current_attendance >= 65.0 {
            "Average"
        } else {
            "Needs Improvement"
        };

        let recommendation = match performance {
            "Excellent" => "Maintain your current study habits and attendance",
            "Good" => "Consider increasing study hours slightly for even better results",
            "Average" => "Focus on improving both study consistency and class attendance",
            "Needs Improvement" => "Significant improvement needed in study hours and attendance",
            _ => "Focus on consistent study habits",
        };

        StudentTrend {
            student_name: student_name.to_string(),
            trend: trend.to_string(),
            current_performance: performance.to_string(),
            recommendation: recommendation.to_string(),
        }
    }

    pub fn generate_class_trends(&self, _student_data: HashMap<String, Vec<(f64, f64)>>) -> ClassTrends {
        // For demo purposes, generate mock trends
        let weeks = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let avg_study_hours = vec![5.2, 5.5, 5.8, 6.1, 6.3, 6.5, 6.7, 6.8];
        let avg_attendance = vec![75.0, 76.5, 78.0, 79.0, 80.0, 81.0, 81.5, 82.0];
        let pass_rates = vec![0.65, 0.68, 0.72, 0.75, 0.77, 0.79, 0.81, 0.82];

        ClassTrends {
            weeks,
            avg_study_hours,
            avg_attendance,
            pass_rates,
        }
    }
}

// Mock data generator
pub fn generate_mock_trends_data() -> HashMap<String, Vec<(f64, f64)>> {
    let mut data = HashMap::new();
    
    let students = vec![
        ("Denis Lemayian".to_string(), vec![(4.5, 70.0), (5.0, 75.0), (5.5, 80.0), (6.0, 85.0)]),
        ("Saitoti Smith".to_string(), vec![(6.0, 85.0), (6.5, 88.0), (7.0, 90.0), (7.5, 92.0)]),
        ("Kukutia Johnson".to_string(), vec![(3.0, 60.0), (3.5, 65.0), (4.0, 70.0), (4.5, 72.0)]),
        ("Kirionki Williams".to_string(), vec![(5.5, 78.0), (5.0, 75.0), (4.5, 72.0), (4.0, 68.0)]),
        ("David Lemoita".to_string(), vec![(7.0, 88.0), (7.5, 90.0), (8.0, 92.0), (8.5, 94.0)]),
    ];

    for (name, weekly_data) in students {
        data.insert(name, weekly_data);
    }

    data
}