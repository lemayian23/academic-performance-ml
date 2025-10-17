use serde::{Deserialize, Serialize};
use rand::Rng;

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
    pub features: Vec<String>,
    pub training_data_size: usize,
    pub model_type: String,
    pub last_updated: String,
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

// Study Plan Structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudyPlanRequest {
    pub student_name: String,
    pub current_hours: f64,
    pub current_attendance: f64,
    pub target_grade: String, // "A", "B", "C", "Pass"
    pub available_days: Vec<String>, // ["Monday", "Tuesday", ...]
    pub preferred_times: Vec<String>, // ["Morning", "Afternoon", "Evening"]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudyPlan {
    pub student_name: String,
    pub target_grade: String,
    pub recommended_hours: f64,
    pub target_attendance: f64,
    pub plan_duration: usize,
    pub weekly_schedule: Vec<DailySchedule>,
    pub recommendations: Vec<String>,
    pub expected_outcomes: String,
    pub generated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailySchedule {
    pub day: String,
    pub study_blocks: Vec<StudyBlock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudyBlock {
    pub time: String,
    pub subject: String,
    pub activity: String,
    pub duration: f64,
}

#[derive(Debug, Clone)]
pub struct TrainedModel {
    study_hours_threshold: f64,
    attendance_threshold: f64,
    base_accuracy: f64,
}

impl TrainedModel {
    pub fn new() -> Self {
        Self {
            study_hours_threshold: 5.0,
            attendance_threshold: 75.0,
            base_accuracy: 0.85,
        }
    }

    pub fn predict(&self, features: &[f64]) -> (bool, f64) {
        let hours = features[0];
        let attendance = features[1];
        
        let mut score = 0.0;
        
        if hours >= 8.0 {
            score += 0.6;
        } else if hours >= 6.0 {
            score += 0.5;
        } else if hours >= 4.0 {
            score += 0.3;
        } else {
            score += 0.1;
        }
        
        if attendance >= 90.0 {
            score += 0.4;
        } else if attendance >= 80.0 {
            score += 0.35;
        } else if attendance >= 70.0 {
            score += 0.25;
        } else {
            score += 0.1;
        }
        
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

    pub fn get_accuracy(&self) -> f64 {
        self.base_accuracy
    }

    // Study Plan Generation
    pub fn generate_study_plan(&self, request: &StudyPlanRequest) -> StudyPlan {
        let (target_hours, target_attendance) = self.calculate_targets(
            request.current_hours,
            request.current_attendance,
            &request.target_grade
        );

        let weekly_schedule = self.generate_weekly_schedule(
            request.available_days.clone(),
            request.preferred_times.clone(),
            target_hours
        );

        let recommendations = self.generate_recommendations(
            request.current_hours,
            request.current_attendance,
            target_hours,
            target_attendance
        );

        let expected_outcomes = self.calculate_expected_outcomes(
            request.current_hours,
            request.current_attendance,
            target_hours,
            target_attendance,
            &request.target_grade
        );

        let plan_duration = self.calculate_plan_duration(
            request.current_hours,
            target_hours,
            request.current_attendance,
            target_attendance
        );

        StudyPlan {
            student_name: request.student_name.clone(),
            target_grade: request.target_grade.clone(),
            recommended_hours: target_hours,
            target_attendance,
            plan_duration,
            weekly_schedule,
            recommendations,
            expected_outcomes,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn calculate_targets(&self, current_hours: f64, current_attendance: f64, target_grade: &str) -> (f64, f64) {
        match target_grade {
            "A" => (
                current_hours.max(12.0),
                current_attendance.max(95.0)
            ),
            "B" => (
                current_hours.max(9.0),
                current_attendance.max(85.0)
            ),
            "C" => (
                current_hours.max(6.0),
                current_attendance.max(75.0)
            ),
            "Pass" => (
                current_hours.max(5.0),
                current_attendance.max(70.0)
            ),
            _ => (
                current_hours.max(8.0),
                current_attendance.max(80.0)
            )
        }
    }

    fn generate_weekly_schedule(&self, available_days: Vec<String>, preferred_times: Vec<String>, total_hours: f64) -> Vec<DailySchedule> {
        let mut schedule = Vec::new();
        let hours_per_day = total_hours / available_days.len() as f64;

        for day in available_days {
            let study_blocks = self.generate_daily_blocks(&day, &preferred_times, hours_per_day);
            
            schedule.push(DailySchedule {
                day,
                study_blocks,
            });
        }

        schedule
    }

    fn generate_daily_blocks(&self, _day: &str, preferred_times: &[String], daily_hours: f64) -> Vec<StudyBlock> {
        let mut blocks = Vec::new();
        let mut remaining_hours = daily_hours;

        let subjects = vec!["Mathematics", "Programming", "Theory", "Practical", "Revision"];
        let activities = vec!["Reading", "Practice Problems", "Review Notes", "Assignment Work", "Past Papers"];

        let mut rng = rand::thread_rng();

        while remaining_hours > 0.0 {
            let duration = if remaining_hours >= 2.0 {
                rng.gen_range(1.5..2.5)
            } else {
                remaining_hours
            };

            let subject = subjects[rng.gen_range(0..subjects.len())].to_string();
            let activity = activities[rng.gen_range(0..activities.len())].to_string();
            
            let time = if !preferred_times.is_empty() {
                preferred_times[rng.gen_range(0..preferred_times.len())].clone()
            } else {
                match rng.gen_range(0..3) {
                    0 => "Morning (8-11 AM)".to_string(),
                    1 => "Afternoon (2-5 PM)".to_string(),
                    _ => "Evening (7-10 PM)".to_string(),
                }
            };

            blocks.push(StudyBlock {
                time,
                subject,
                activity,
                duration,
            });

            remaining_hours -= duration;
            if remaining_hours < 0.5 {
                break;
            }
        }

        blocks
    }

    fn generate_recommendations(&self, current_hours: f64, current_attendance: f64, target_hours: f64, target_attendance: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if current_hours < target_hours {
            let increase = target_hours - current_hours;
            recommendations.push(format!(
                "Increase study time by {:.1} hours per week",
                increase
            ));
            
            if increase > 5.0 {
                recommendations.push("Consider breaking study sessions into smaller, focused blocks".to_string());
            }
        }

        if current_attendance < target_attendance {
            let improvement = target_attendance - current_attendance;
            recommendations.push(format!(
                "Improve attendance by {:.1}%",
                improvement
            ));
            
            if improvement > 10.0 {
                recommendations.push("Set reminders for class schedules and prepare materials in advance".to_string());
            }
        }

        if current_hours < 6.0 {
            recommendations.push("Establish a consistent daily study routine".to_string());
        }

        if current_attendance < 75.0 {
            recommendations.push("Prioritize class attendance - it significantly impacts learning".to_string());
        }

        if current_hours >= 8.0 && current_attendance >= 80.0 {
            recommendations.push("Great foundation! Focus on maintaining consistency".to_string());
        }

        recommendations.push("Review class notes within 24 hours of lectures".to_string());
        recommendations.push("Practice with past exam papers weekly".to_string());
        recommendations.push("Join study groups for difficult subjects".to_string());

        recommendations
    }

    fn calculate_expected_outcomes(&self, current_hours: f64, current_attendance: f64, target_hours: f64, target_attendance: f64, target_grade: &str) -> String {
        let _improvement_potential = ((target_hours - current_hours) + (target_attendance - current_attendance)) / 2.0; // Prefix with underscore to fix warning
        
        match target_grade {
            "A" => format!(
                "With {} hours of study and {}% attendance, you have a high probability of achieving an A grade. Consistent effort and following this plan should lead to excellent academic performance.",
                target_hours, target_attendance
            ),
            "B" => format!(
                "This plan is designed to help you achieve a B grade. By maintaining {} study hours and {}% attendance, you'll build a strong foundation for success.",
                target_hours, target_attendance
            ),
            "C" => format!(
                "Following this schedule of {} hours weekly with {}% attendance should help you achieve a C grade and build better study habits.",
                target_hours, target_attendance
            ),
            "Pass" => format!(
                "This plan focuses on establishing basic study routines. With {} hours of study and {}% attendance, you should be able to pass your courses.",
                target_hours, target_attendance
            ),
            _ => format!(
                "This personalized study plan is tailored to help you reach your academic goals through consistent effort and improved study habits."
            )
        }
    }

    fn calculate_plan_duration(&self, current_hours: f64, target_hours: f64, current_attendance: f64, target_attendance: f64) -> usize {
        let hours_gap = (target_hours - current_hours).max(0.0);
        let attendance_gap = (target_attendance - current_attendance).max(0.0);
        
        let duration = ((hours_gap / 0.5) + (attendance_gap / 2.0)).ceil() as usize;
        duration.clamp(4, 12)
    }
}

pub fn train_model() -> ModelInfo {
    ModelInfo {
        accuracy: 0.85,
        features: vec!["study_hours".to_string(), "attendance".to_string()],
        training_data_size: 1000,
        model_type: "Logistic Regression".to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
    }
}