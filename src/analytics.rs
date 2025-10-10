use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct StudentTrend {
    pub student_name: String,
    pub weekly_data: Vec<WeekData>,
    pub overall_trend: String,
    pub improvement_score: f64,
}

#[derive(Serialize)]
pub struct WeekData {
    pub week: usize,
    pub study_hours: f64,
    pub attendance: f64,
    pub predicted_pass: bool,
    pub confidence: f64,
}

#[derive(Serialize)]
pub struct ClassTrends {
    pub total_students: usize,
    pub weekly_summary: Vec<WeekSummary>,
    pub top_performers: Vec<String>,
    pub at_risk_students: Vec<String>,
    pub average_improvement: f64,
}

#[derive(Serialize)]
pub struct WeekSummary {
    pub week: usize,
    pub avg_study_hours: f64,
    pub avg_attendance: f64,
    pub pass_rate: f64,
    pub total_predictions: usize,
}

pub struct TrendsAnalyzer;

impl TrendsAnalyzer {
    pub fn new() -> Self {
        TrendsAnalyzer
    }

    pub fn generate_student_trend(&self, student_name: &str, historical_data: Vec<(f64, f64)>) -> StudentTrend {
        let weekly_data: Vec<WeekData> = historical_data
            .iter()
            .enumerate()
            .map(|(week, &(hours, attendance))| {
                let predicted_pass = hours >= 5.0 && attendance >= 75.0;
                let confidence = ((hours * 0.1) + (attendance * 0.01)).min(1.0);
                
                WeekData {
                    week: week + 1,
                    study_hours: hours,
                    attendance,
                    predicted_pass,
                    confidence,
                }
            })
            .collect();

        let improvement_score = self.calculate_improvement_score(&weekly_data);
        let overall_trend = self.determine_trend(&weekly_data);

        StudentTrend {
            student_name: student_name.to_string(),
            weekly_data,
            overall_trend,
            improvement_score,
        }
    }

    pub fn generate_class_trends(&self, students_data: HashMap<String, Vec<(f64, f64)>>) -> ClassTrends {
        let total_students = students_data.len();
        let mut weekly_summaries = Vec::new();
        let mut student_scores = HashMap::new();

        for week in 1..=4 {
            let mut total_hours = 0.0;
            let mut total_attendance = 0.0;
            let mut pass_count = 0;
            let mut total_predictions = 0;

            for (student_name, data) in &students_data {
                if let Some(&(hours, attendance)) = data.get(week - 1) {
                    total_hours += hours;
                    total_attendance += attendance;
                    total_predictions += 1;

                    if hours >= 5.0 && attendance >= 75.0 {
                        pass_count += 1;
                    }

                    let score = student_scores.entry(student_name.clone()).or_insert(0.0);
                    *score += (hours * 0.5) + (attendance * 0.5);
                }
            }

            weekly_summaries.push(WeekSummary {
                week,
                avg_study_hours: if total_predictions > 0 { total_hours / total_predictions as f64 } else { 0.0 },
                avg_attendance: if total_predictions > 0 { total_attendance / total_predictions as f64 } else { 0.0 },
                pass_rate: if total_predictions > 0 { pass_count as f64 / total_predictions as f64 } else { 0.0 },
                total_predictions,
            });
        }

        let mut sorted_students: Vec<(String, f64)> = student_scores.into_iter().collect();
        sorted_students.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let top_performers: Vec<String> = sorted_students
            .iter()
            .take(3)
            .map(|(name, _)| name.clone())
            .collect();

        let at_risk_students: Vec<String> = sorted_students
            .iter()
            .rev()
            .take(3)
            .map(|(name, _)| name.clone())
            .collect();

        let average_improvement = self.calculate_average_improvement(&weekly_summaries);

        ClassTrends {
            total_students,
            weekly_summary: weekly_summaries,
            top_performers,
            at_risk_students,
            average_improvement,
        }
    }

    fn calculate_improvement_score(&self, weekly_data: &[WeekData]) -> f64 {
        if weekly_data.len() < 2 {
            return 0.0;
        }

        let first_week = &weekly_data[0];
        let last_week = &weekly_data[weekly_data.len() - 1];

        let hours_improvement = last_week.study_hours - first_week.study_hours;
        let attendance_improvement = last_week.attendance - first_week.attendance;

        (hours_improvement * 0.6 + attendance_improvement * 0.4).max(0.0).min(10.0)
    }

    fn determine_trend(&self, weekly_data: &[WeekData]) -> String {
        if weekly_data.len() < 2 {
            return "Stable".to_string();
        }

        let first_confidence = weekly_data[0].confidence;
        let last_confidence = weekly_data[weekly_data.len() - 1].confidence;

        if last_confidence > first_confidence + 0.1 {
            "Improving".to_string()
        } else if last_confidence < first_confidence - 0.1 {
            "Declining".to_string()
        } else {
            "Stable".to_string()
        }
    }

    fn calculate_average_improvement(&self, weekly_summaries: &[WeekSummary]) -> f64 {
        if weekly_summaries.len() < 2 {
            return 0.0;
        }

        let first_week = &weekly_summaries[0];
        let last_week = &weekly_summaries[weekly_summaries.len() - 1];

        let hours_improvement = last_week.avg_study_hours - first_week.avg_study_hours;
        let attendance_improvement = last_week.avg_attendance - first_week.avg_attendance;

        (hours_improvement + attendance_improvement) / 2.0
    }
}

// Mock data generator
pub fn generate_mock_trends_data() -> HashMap<String, Vec<(f64, f64)>> {
    let mut data = HashMap::new();
    
    let students = vec![
        ("Denis Lemayian", vec![(4.5, 70.0), (5.0, 75.0), (5.5, 80.0), (6.0, 85.0)]),
        ("Saitoti Smith", vec![(6.0, 85.0), (6.5, 88.0), (7.0, 90.0), (7.5, 92.0)]),
        ("Kukutia Johnson", vec![(3.0, 60.0), (3.5, 65.0), (4.0, 70.0), (4.5, 72.0)]),
        ("Kirionki Williams", vec![(5.5, 78.0), (5.0, 75.0), (4.5, 72.0), (4.0, 68.0)]),
        ("David Lemoita", vec![(7.0, 88.0), (7.5, 90.0), (8.0, 92.0), (8.5, 94.0)]),
    ];

    for (name, weekly_data) in students {
        data.insert(name.to_string(), weekly_data);
    }

    data
}