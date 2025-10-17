mod data;
mod model;
mod analytics;
mod database;
mod gamification;

use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use rand::Rng;

use crate::model::{train_model, ModelInfo, PredictResponse, AnalyticsData, PerformanceCategory, 
                   StudentRecord as ModelStudentRecord, TrainedModel, StudyPlanRequest};
use crate::analytics::{TrendsAnalyzer, generate_mock_trends_data};
use crate::database::{Database, StudentRecord as DbStudentRecord, ModelVersion};
use crate::gamification::{
    GamificationEngine, StudySessionRequest, 
    GamificationResponse, get_mock_leaderboard, get_mock_profile
};

#[derive(Deserialize)]
struct PredictRequest {
    hours: f64,
    attendance: f64,
}

// Student trends request
#[derive(Deserialize)]
struct StudentTrendsRequest {
    student_name: String,
    weekly_data: Vec<WeeklyData>,
}

#[derive(Deserialize)]
struct WeeklyData {
    week: usize,
    study_hours: f64,
    attendance: f64,
}

// Student progress tracking request
#[derive(Deserialize)]
struct ProgressRequest {
    student_name: String,
    weeks: usize,
}

// Student trends endpoint
async fn get_student_trends(
    req: web::Json<StudentTrendsRequest>,
) -> HttpResponse {
    let analyzer = TrendsAnalyzer::new();
    
    let historical_data: Vec<(f64, f64)> = req.weekly_data
        .iter()
        .map(|wd| (wd.study_hours, wd.attendance))
        .collect();

    let trend = analyzer.generate_student_trend(&req.student_name, historical_data);
    HttpResponse::Ok().json(trend)
}

// Class trends endpoint
async fn get_class_trends() -> HttpResponse {
    let analyzer = TrendsAnalyzer::new();
    let mock_data = generate_mock_trends_data();
    let class_trends = analyzer.generate_class_trends(mock_data);
    HttpResponse::Ok().json(class_trends)
}

// Trends dashboard endpoint
async fn get_trends_dashboard() -> HttpResponse {
    let analyzer = TrendsAnalyzer::new();
    let mock_data = generate_mock_trends_data();
    
    let class_trends = analyzer.generate_class_trends(mock_data.clone());
    
    let mut student_trends = Vec::new();
    for (student_name, data) in mock_data.iter().take(3) {
        let trend = analyzer.generate_student_trend(student_name, data.clone());
        student_trends.push(trend);
    }

    let dashboard_data = serde_json::json!({
        "class_trends": class_trends,
        "student_trends": student_trends,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    HttpResponse::Ok().json(dashboard_data)
}

// Student progress tracking endpoint
async fn track_student_progress(
    req: web::Json<ProgressRequest>,
    model: web::Data<TrainedModel>,
) -> HttpResponse {
    let mut progress_data = Vec::new();
    let mut rng = rand::thread_rng();
    
    // Generate simulated progress data
    for week in 1..=req.weeks {
        // Simulate gradual improvement
        let base_hours = 4.0 + (week as f64 * 0.5);
        let base_attendance = 70.0 + (week as f64 * 2.0);
        
        // Add some randomness
        let hours = (base_hours + rng.gen_range(-1.0..1.0)).max(0.0);
        let attendance = (base_attendance + rng.gen_range(-5.0..5.0)).clamp(0.0, 100.0);
        
        let features = vec![hours, attendance];
        let (prediction, confidence) = model.predict(&features);
        
        progress_data.push(serde_json::json!({
            "week": week,
            "study_hours": hours,
            "attendance": attendance,
            "prediction": if prediction { "Pass" } else { "Fail" },
            "confidence": confidence,
            "improvement_score": (hours * 0.6 + attendance * 0.4) / 100.0 * 10.0
        }));
    }
    
    let response = serde_json::json!({
        "student_name": req.student_name,
        "tracking_weeks": req.weeks,
        "progress_data": progress_data,
        "overall_trend": analyze_progress_trend(&progress_data),
        "generated_at": chrono::Utc::now().to_rfc3339()
    });
    
    HttpResponse::Ok().json(response)
}

fn analyze_progress_trend(progress_data: &[serde_json::Value]) -> String {
    if progress_data.len() < 2 {
        return "Insufficient data".to_string();
    }
    
    let first_score = progress_data[0]["improvement_score"].as_f64().unwrap_or(0.0);
    let last_score = progress_data.last().unwrap()["improvement_score"].as_f64().unwrap_or(0.0);
    
    if last_score > first_score + 1.0 {
        "Improving".to_string()
    } else if last_score < first_score - 1.0 {
        "Declining".to_string()
    } else {
        "Stable".to_string()
    }
}

// Study Plan Generator endpoint
async fn generate_study_plan(
    req: web::Json<StudyPlanRequest>,
    model: web::Data<TrainedModel>,
) -> HttpResponse {
    let study_plan = model.generate_study_plan(&req);
    HttpResponse::Ok().json(study_plan)
}

// Prediction endpoint with database
async fn predict(
    req: web::Json<PredictRequest>,
    model: web::Data<TrainedModel>,
    db: web::Data<Database>,
) -> HttpResponse {
    let features = vec![req.hours, req.attendance];
    let (prediction, confidence) = model.predict(&features);
    
    // Save to database
    let record = DbStudentRecord {
        id: 0,
        name: "Anonymous Student".to_string(),
        study_hours: req.hours,
        attendance: req.attendance,
        predicted_pass: prediction,
        confidence,
        created_at: chrono::Utc::now(),
    };

    if let Err(e) = db.save_prediction(&record).await {
        eprintln!("Failed to save prediction: {}", e);
    }

    let response = PredictResponse {
        prediction: if prediction { "Pass".to_string() } else { "Fail".to_string() },
        confidence,
    };
    
    HttpResponse::Ok().json(response)
}

// Batch prediction endpoint with database
async fn batch_predict(
    students: web::Json<Vec<ModelStudentRecord>>,
    model: web::Data<TrainedModel>,
    db: web::Data<Database>,
) -> HttpResponse {
    let batch_result = model.batch_predict(students.into_inner());
    
    // Save batch predictions to database
    for student in &batch_result.predictions {
        let record = DbStudentRecord {
            id: 0,
            name: student.name.clone(),
            study_hours: student.hours,
            attendance: student.attendance,
            predicted_pass: student.prediction == "Pass",
            confidence: student.confidence,
            created_at: chrono::Utc::now(),
        };

        if let Err(e) = db.save_prediction(&record).await {
            eprintln!("Failed to save batch prediction for {}: {}", student.name, e);
        }
    }
    
    HttpResponse::Ok().json(batch_result)
}

// Analytics endpoint
async fn get_analytics() -> HttpResponse {
    let analytics = AnalyticsData {
        total_students: 150,
        pass_rate: 0.72,
        avg_study_hours: 5.8,
        avg_attendance: 78.5,
        performance_breakdown: vec![
            PerformanceCategory {
                range: "Excellent (90-100%)".to_string(),
                count: 45,
                pass_rate: 0.95,
            },
            PerformanceCategory {
                range: "Good (75-89%)".to_string(),
                count: 62,
                pass_rate: 0.82,
            },
            PerformanceCategory {
                range: "Average (60-74%)".to_string(),
                count: 35,
                pass_rate: 0.45,
            },
            PerformanceCategory {
                range: "Needs Improvement (<60%)".to_string(),
                count: 8,
                pass_rate: 0.15,
            },
        ],
    };
    
    HttpResponse::Ok().json(analytics)
}

// Database statistics endpoint
async fn get_database_analytics(db: web::Data<Database>) -> HttpResponse {
    match db.get_class_statistics().await {
        Ok(stats) => {
            let analytics = AnalyticsData {
                total_students: stats.total_students as usize,
                pass_rate: stats.pass_rate,
                avg_study_hours: stats.avg_study_hours,
                avg_attendance: stats.avg_attendance,
                performance_breakdown: vec![
                    PerformanceCategory {
                        range: "Real Data".to_string(),
                        count: stats.total_students as usize,
                        pass_rate: stats.pass_rate,
                    },
                ],
            };
            HttpResponse::Ok().json(analytics)
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Real trends from database
async fn get_real_trends_dashboard(db: web::Data<Database>) -> HttpResponse {
    match db.get_weekly_trends().await {
        Ok(weekly_trends) => {
            let analyzer = TrendsAnalyzer::new();
            
            // Convert database trends to analytics format
            let mut student_trends = Vec::new();
            
            // For real trends, we'll use the weekly aggregated data
            for (_index, trend) in weekly_trends.iter().enumerate() {
                let student_name = format!("Week {}", trend.week);
                let historical_data = vec![
                    (trend.avg_study_hours, trend.avg_attendance)
                ];
                let student_trend = analyzer.generate_student_trend(&student_name, historical_data);
                student_trends.push(student_trend);
            }
            
            // Generate mock class trends for now since we need individual student data
            let mock_data = generate_mock_trends_data();
            let class_trends = analyzer.generate_class_trends(mock_data);
            
            let dashboard_data = serde_json::json!({
                "class_trends": class_trends,
                "student_trends": student_trends,
                "weekly_trends": weekly_trends,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "data_source": "real_database",
                "total_records": weekly_trends.len(),
            });

            HttpResponse::Ok().json(dashboard_data)
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Get all predictions from database
async fn get_all_predictions(db: web::Data<Database>) -> HttpResponse {
    match db.get_all_predictions().await {
        Ok(predictions) => HttpResponse::Ok().json(predictions),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}

// Save model version to database
async fn save_model_version(
    model_info: web::Data<ModelInfo>,
    db: web::Data<Database>,
) -> HttpResponse {
    let model_version = ModelVersion {
        id: 0,
        version: "1.0.0".to_string(),
        accuracy: model_info.accuracy,
        features_used: "study_hours,attendance".to_string(),
        created_at: chrono::Utc::now(),
    };

    match db.save_model_version(&model_version).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Model version saved successfully",
            "version": model_version
        })),
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to save model version: {}", e)
            }))
        }
    }
}

// Model info endpoint
async fn get_model_info(model_data: web::Data<ModelInfo>) -> HttpResponse {
    HttpResponse::Ok().json(model_data.as_ref().clone())
}

// Health check endpoint
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("TUK Student Classifier API is running!")
}

// Success tips endpoint
async fn get_success_tips() -> HttpResponse {
    let tips = vec![
        "🎯 Study at least 5 hours weekly for better results",
        "📚 Maintain 80%+ attendance for higher pass rates", 
        "⏰ Consistent daily study beats last-minute cramming",
        "📝 Practice with past papers regularly",
        "🔄 Review class notes within 24 hours",
        "👥 Join study groups for difficult subjects",
        "💤 Get 7-8 hours sleep for optimal memory retention"
    ];
    
    HttpResponse::Ok().json(tips)
}

// Gamification endpoints
async fn record_study_session(
    req: web::Json<StudySessionRequest>,
    gamification: web::Data<GamificationEngine>,
) -> HttpResponse {
    let points_earned = gamification.calculate_points(&req);
    
    // In a real app, you'd save this to a database
    let mock_profile = get_mock_profile(&req.student_name);
    let new_badges = gamification.check_badges(&mock_profile, &req);
    let new_achievements = gamification.check_achievements(&mock_profile, &req);
    let _new_streak = gamification.update_streak(&mock_profile, &req);
    let new_level = gamification.calculate_level(mock_profile.total_points + points_earned);
    let level_up = new_level > mock_profile.level;

    let response = GamificationResponse {
        profile: mock_profile,
        points_earned,
        level_up,
        new_badges,
        new_achievements,
        leaderboard_position: 3, // Mock position
    };

    HttpResponse::Ok().json(response)
}

async fn get_leaderboard() -> HttpResponse {
    let leaderboard = get_mock_leaderboard();
    HttpResponse::Ok().json(leaderboard)
}

async fn get_student_profile(
    path: web::Path<String>,
) -> HttpResponse {
    let student_name = path.into_inner();
    let profile = get_mock_profile(&student_name);
    HttpResponse::Ok().json(profile)
}

async fn get_achievements(
    path: web::Path<String>,
) -> HttpResponse {
    let student_name = path.into_inner();
    let profile = get_mock_profile(&student_name);
    HttpResponse::Ok().json(profile.achievements)
}

async fn get_badges(
    path: web::Path<String>,
) -> HttpResponse {
    let student_name = path.into_inner();
    let profile = get_mock_profile(&student_name);
    HttpResponse::Ok().json(profile.badges)
}

// Homepage endpoint with complete HTML including all features
async fn serve_homepage() -> HttpResponse {
    let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>The Technical University of Kenya - Student Performance Predictor</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 20px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
        }

        .header {
            text-align: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 3px solid #007bff;
            background: linear-gradient(135deg, #007bff, #0056b3);
            color: white;
            padding: 30px;
            border-radius: 15px;
            margin-top: -10px;
        }

        .header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }

        .header h2 {
            font-size: 1.5em;
            opacity: 0.9;
            font-weight: 300;
        }

        .form-group {
            margin: 25px 0;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 12px;
            border-left: 4px solid #007bff;
        }

        label {
            display: block;
            margin-bottom: 12px;
            font-weight: 600;
            color: #495057;
            font-size: 1.1em;
        }

        input, select {
            width: 100%;
            padding: 15px;
            border: 2px solid #e9ecef;
            border-radius: 10px;
            font-size: 16px;
            transition: all 0.3s ease;
            background: white;
        }

        input:focus, select:focus {
            border-color: #007bff;
            outline: none;
            box-shadow: 0 0 0 3px rgba(0,123,255,0.1);
            transform: translateY(-2px);
        }

        button {
            background: linear-gradient(135deg, #007bff, #0056b3);
            color: white;
            padding: 16px 32px;
            border: none;
            border-radius: 10px;
            cursor: pointer;
            margin: 8px;
            font-size: 16px;
            font-weight: 600;
            transition: all 0.3s ease;
            box-shadow: 0 4px 15px rgba(0,123,255,0.3);
        }

        button:hover {
            transform: translateY(-3px);
            box-shadow: 0 8px 25px rgba(0,123,255,0.4);
        }

        .result {
            margin-top: 25px;
            padding: 25px;
            border-radius: 12px;
            display: none;
            border-left: 5px solid;
            animation: slideIn 0.5s ease;
        }

        @keyframes slideIn {
            from { opacity: 0; transform: translateY(-20px); }
            to { opacity: 1; transform: translateY(0); }
        }

        .pass { 
            background: #d4edda; 
            color: #155724; 
            border-left-color: #28a745; 
        }

        .fail { 
            background: #f8d7da; 
            color: #721c24; 
            border-left-color: #dc3545; 
        }

        .info { 
            background: #d1ecf1; 
            color: #0c5460; 
            border-left-color: #17a2b8; 
        }

        .warning { 
            background: #fff3cd; 
            color: #856404; 
            border-left-color: #ffc107; 
        }

        .success { 
            background: #e8f5e8; 
            color: #155724; 
            border-left-color: #28a745; 
        }

        .button-group {
            text-align: center;
            margin: 30px 0;
            display: flex;
            justify-content: center;
            flex-wrap: wrap;
            gap: 10px;
        }

        .feature-section {
            background: #e8f5e8;
            padding: 25px;
            border-radius: 12px;
            margin: 25px 0;
            border: 2px solid #28a745;
        }

        .trends-section {
            background: #fff3e6;
            padding: 25px;
            border-radius: 12px;
            margin: 25px 0;
            border: 2px solid #fd7e14;
        }

        .database-section {
            background: #e6f3ff;
            padding: 25px;
            border-radius: 12px;
            margin: 25px 0;
            border: 2px solid #007bff;
        }

        .progress-section {
            background: #f0e6ff;
            padding: 25px;
            border-radius: 12px;
            margin: 25px 0;
            border: 2px solid #6f42c1;
        }

        .study-plan-section {
            background: #fff0f6;
            padding: 25px;
            border-radius: 12px;
            margin: 25px 0;
            border: 2px solid #e83e8c;
        }

        .gamification-section {
            background: linear-gradient(135deg, #ffd700, #ff6b6b);
            padding: 25px;
            border-radius: 12px;
            margin: 25px 0;
            border: 2px solid #ff8c00;
        }

        .prediction-table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .prediction-table th {
            background: #343a40;
            color: white;
            padding: 15px;
            text-align: left;
        }

        .prediction-table td {
            padding: 12px;
            border-bottom: 1px solid #dee2e6;
        }

        .prediction-table tr:hover {
            background: #f8f9fa;
        }

        .chart-container {
            background: white;
            padding: 20px;
            border-radius: 10px;
            margin: 20px 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .chart-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin: 20px 0;
        }

        @media (max-width: 768px) {
            .chart-grid {
                grid-template-columns: 1fr;
            }
            .button-group {
                flex-direction: column;
                align-items: center;
            }
            button {
                width: 100%;
                max-width: 300px;
            }
        }

        .trend-card {
            background: white;
            padding: 20px;
            margin: 15px 0;
            border-radius: 10px;
            border-left: 5px solid;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .improving { border-left-color: #28a745; }
        .declining { border-left-color: #dc3545; }
        .stable { border-left-color: #ffc107; }

        .metric-card {
            background: white;
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            border-top: 4px solid;
        }

        .metric-value {
            font-size: 2.5em;
            font-weight: bold;
            margin: 10px 0;
        }

        .tab-container {
            margin: 20px 0;
        }

        .tab-buttons {
            display: flex;
            border-bottom: 1px solid #dee2e6;
        }

        .tab-button {
            padding: 12px 24px;
            background: none;
            border: none;
            cursor: pointer;
            border-bottom: 3px solid transparent;
            transition: all 0.3s ease;
        }

        .tab-button.active {
            border-bottom-color: #007bff;
            color: #007bff;
            font-weight: bold;
        }

        .tab-content {
            display: none;
            padding: 20px 0;
        }

        textarea {
            width: 100%;
            padding: 15px;
            border: 2px solid #e9ecef;
            border-radius: 10px;
            font-size: 16px;
            font-family: 'Courier New', monospace;
            resize: vertical;
            min-height: 120px;
        }

        textarea:focus {
            border-color: #007bff;
            outline: none;
            box-shadow: 0 0 0 3px rgba(0,123,255,0.1);
        }

        .checkbox-group {
            display: flex;
            flex-wrap: wrap;
            gap: 15px;
            margin: 10px 0;
        }

        .checkbox-item {
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .schedule-day {
            background: white;
            padding: 20px;
            border-radius: 10px;
            margin: 15px 0;
            border-left: 4px solid #007bff;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .study-block {
            background: #f8f9fa;
            padding: 15px;
            margin: 10px 0;
            border-radius: 8px;
            border-left: 3px solid #28a745;
        }

        .recommendation-item {
            background: white;
            padding: 15px;
            margin: 10px 0;
            border-radius: 8px;
            border-left: 3px solid #ffc107;
        }

        .badge-item {
            background: white;
            padding: 15px;
            margin: 10px 0;
            border-radius: 10px;
            border-left: 4px solid gold;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .achievement-item {
            background: white;
            padding: 15px;
            margin: 10px 0;
            border-radius: 10px;
            border-left: 4px solid #28a745;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎓 The Technical University of Kenya</h1>
            <h2>Student Performance Analytics Dashboard</h2>
            <p style="margin-top: 10px; opacity: 0.8;">AI-Powered Student Success Prediction System</p>
        </div>
        
        <div class="form-group">
            <label for="hours">📚 Weekly Study Hours:</label>
            <input type="number" id="hours" step="0.1" placeholder="e.g., 6.0" value="6.0" min="0" max="168">
        </div>
        
        <div class="form-group">
            <label for="attendance">🏫 Class Attendance (%):</label>
            <input type="number" id="attendance" step="0.1" placeholder="e.g., 85.0" value="85.0" min="0" max="100">
        </div>
        
        <div style="text-align: center;">
            <button onclick="predict()" style="background: linear-gradient(135deg, #28a745, #20c997); font-size: 18px; padding: 18px 36px;">
                📊 Predict Academic Result
            </button>
        </div>
        
        <div id="result" class="result"></div>

        <div class="button-group">
            <button onclick="showAnalytics()" style="background: #28a745;">📈 Performance Analytics</button>
            <button onclick="showDatabaseAnalytics()" style="background: #17a2b8;">💾 Real Database Analytics</button>
            <button onclick="showTips()" style="background: #6f42c1;">💡 Success Tips</button>
            <button onclick="showModelInfo()" style="background: #fd7e14;">🤖 Model Info</button>
            <button onclick="loadTrendsDashboard()" style="background: #e83e8c;">📊 Interactive Dashboard</button>
            <button onclick="showAllPredictions()" style="background: #6f42c1;">🗃️ View All Predictions</button>
        </div>

        <!-- Gamification Section -->
        <div class="gamification-section">
            <h3 style="color: #8b4513; margin-top: 0;">🎮 Study Gamification System</h3>
            <p>Earn points, badges, and climb the leaderboard by studying consistently!</p>
            
            <div class="form-group">
                <label for="sessionStudentName">👨‍🎓 Student Name:</label>
                <input type="text" id="sessionStudentName" placeholder="e.g., Denis Lemayian" value="Denis Lemayian">
            </div>
            
            <div class="form-group">
                <label for="sessionDuration">⏱️ Study Duration (hours):</label>
                <input type="number" id="sessionDuration" step="0.1" placeholder="e.g., 2.5" value="2.5" min="0.5" max="12">
            </div>
            
            <div class="form-group">
                <label for="sessionSubjects">📚 Subjects Studied (comma-separated):</label>
                <input type="text" id="sessionSubjects" placeholder="e.g., Mathematics, Programming" value="Mathematics, Programming">
            </div>
            
            <div class="form-group">
                <label for="focusScore">🎯 Focus Score (0-100%):</label>
                <input type="number" id="focusScore" step="1" placeholder="e.g., 85" value="85" min="0" max="100">
            </div>
            
            <div class="checkbox-item">
                <input type="checkbox" id="attendanceToday" checked>
                <label for="attendanceToday">✅ Attended classes today</label>
            </div>
            
            <div style="text-align: center; margin-top: 20px;">
                <button onclick="recordStudySession()" style="background: linear-gradient(135deg, #ff8c00, #ff6347); font-size: 18px; padding: 18px 36px;">
                    🎮 Record Study Session
                </button>
                <button onclick="showLeaderboard()" style="background: linear-gradient(135deg, #9370db, #8a2be2);">
                    🏆 View Leaderboard
                </button>
                <button onclick="showMyProfile()" style="background: linear-gradient(135deg, #20b2aa, #008080);">
                    👤 My Profile
                </button>
            </div>
            
            <div id="gamification-result" class="result" style="display: none;"></div>
            <div id="leaderboard-result" class="result" style="display: none;"></div>
            <div id="profile-result" class="result" style="display: none;"></div>
        </div>

        <!-- Study Plan Generator Section -->
        <div class="study-plan-section">
            <h3 style="color: #e83e8c; margin-top: 0;">📚 Personalized Study Plan Generator</h3>
            <p>Create a customized study schedule based on your academic goals:</p>
            
            <div class="form-group">
                <label for="planStudentName">👨‍🎓 Student Name:</label>
                <input type="text" id="planStudentName" placeholder="e.g., John Doe" value="Denis Lemayian">
            </div>
            
            <div class="form-group">
                <label for="currentHours">📚 Current Weekly Study Hours:</label>
                <input type="number" id="currentHours" step="0.1" placeholder="e.g., 6.0" value="6.0" min="0" max="168">
            </div>
            
            <div class="form-group">
                <label for="currentAttendance">🏫 Current Attendance (%):</label>
                <input type="number" id="currentAttendance" step="0.1" placeholder="e.g., 75.0" value="75.0" min="0" max="100">
            </div>
            
            <div class="form-group">
                <label for="targetGrade">🎯 Target Grade:</label>
                <select id="targetGrade">
                    <option value="A">A - Excellent (90-100%)</option>
                    <option value="B">B - Good (75-89%)</option>
                    <option value="C" selected>C - Average (60-74%)</option>
                    <option value="Pass">Pass (50-59%)</option>
                </select>
            </div>
            
            <div class="form-group">
                <label>📅 Available Study Days:</label>
                <div class="checkbox-group">
                    <div class="checkbox-item">
                        <input type="checkbox" id="monday" checked>
                        <label for="monday">Monday</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="tuesday" checked>
                        <label for="tuesday">Tuesday</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="wednesday" checked>
                        <label for="wednesday">Wednesday</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="thursday" checked>
                        <label for="thursday">Thursday</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="friday" checked>
                        <label for="friday">Friday</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="saturday">
                        <label for="saturday">Saturday</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="sunday">
                        <label for="sunday">Sunday</label>
                    </div>
                </div>
            </div>
            
            <div class="form-group">
                <label>⏰ Preferred Study Times:</label>
                <div class="checkbox-group">
                    <div class="checkbox-item">
                        <input type="checkbox" id="morning" checked>
                        <label for="morning">Morning (8-11 AM)</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="afternoon" checked>
                        <label for="afternoon">Afternoon (2-5 PM)</label>
                    </div>
                    <div class="checkbox-item">
                        <input type="checkbox" id="evening" checked>
                        <label for="evening">Evening (7-10 PM)</label>
                    </div>
                </div>
            </div>
            
            <div style="text-align: center;">
                <button onclick="generateStudyPlan()" style="background: linear-gradient(135deg, #e83e8c, #6f42c1); font-size: 18px; padding: 18px 36px;">
                    🎓 Generate Study Plan
                </button>
            </div>
            
            <div id="study-plan-result" class="result" style="display: none;"></div>
        </div>

        <!-- Student Progress Tracking Section -->
        <div class="progress-section">
            <h3 style="color: #6f42c1; margin-top: 0;">📈 Student Progress Tracking</h3>
            <p>Track student improvement over multiple weeks:</p>
            
            <div class="form-group">
                <label for="studentName">👨‍🎓 Student Name:</label>
                <input type="text" id="studentName" placeholder="e.g., John Doe" value="Denis Lemayian">
            </div>
            
            <div class="form-group">
                <label for="trackingWeeks">📅 Weeks to Track:</label>
                <input type="number" id="trackingWeeks" placeholder="e.g., 8" value="8" min="1" max="52">
            </div>
            
            <div style="text-align: center;">
                <button onclick="trackProgress()" style="background: linear-gradient(135deg, #6f42c1, #e83e8c);">
                    📊 Generate Progress Report
                </button>
            </div>
            
            <div id="progress-result" class="result" style="display: none;"></div>
        </div>

        <!-- Database Analytics Section -->
        <div class="database-section">
            <h3 style="color: #007bff; margin-top: 0;">💾 Real Database Analytics</h3>
            <p>View analytics based on actual predictions stored in the database:</p>
            
            <div id="database-analytics" class="result" style="display: none;"></div>
            <div id="all-predictions" class="result" style="display: none;"></div>
        </div>

        <!-- Trends Dashboard Section -->
        <div class="trends-section">
            <h3 style="color: #e83e8c; margin-top: 0;">📈 Advanced Analytics Dashboard</h3>
            <p>Professional-grade performance tracking with interactive visualizations:</p>
            
            <div class="tab-container">
                <div class="tab-buttons">
                    <button class="tab-button active" onclick="switchTab('mock-dashboard')">Mock Data Dashboard</button>
                    <button class="tab-button" onclick="switchTab('real-dashboard')">Real Data Dashboard</button>
                </div>
                
                <div id="mock-dashboard" class="tab-content">
                    <div id="trends-dashboard" class="result" style="display: none;"></div>
                </div>
                
                <div id="real-dashboard" class="tab-content" style="display: none;">
                    <div id="real-trends-dashboard" class="result" style="display: none;"></div>
                </div>
            </div>
            
            <div id="class-trends" class="result" style="display: none;"></div>
        </div>

        <!-- Batch Prediction Section -->
        <div class="feature-section">
            <h3 style="color: #28a745; margin-top: 0;">📁 Batch Student Prediction</h3>
            <p>Upload multiple students for bulk analysis (saved to database):</p>
            
            <textarea id="batchData" placeholder="Enter CSV data:
name,hours,attendance
Denis Lemayian,6.5,85.0
Saitoti Smith,4.0,70.0
Kukutia Johnson,8.0,92.0
Kirionki Williams,3.0,65.0
David Lemoita,7.5,88.0" rows="8"></textarea>
            
            <div style="text-align: center; margin-top: 15px;">
                <button onclick="processBatch()" style="background: #28a745;">📊 Process Batch Predictions</button>
            </div>
            
            <div id="batch-result" class="result" style="display: none;"></div>
        </div>

        <div id="analytics" class="result info" style="display: none;">
            <h3>📈 TUK Student Performance Analytics</h3>
            <div id="analytics-content"></div>
        </div>

        <div id="tips" class="result info" style="display: none;">
            <h3>💡 TUK Student Success Tips</h3>
            <div id="tips-content"></div>
        </div>

        <div id="model-info" class="result info" style="display: none;">
            <h3>🤖 Model Information</h3>
            <div id="model-info-content"></div>
        </div>
    </div>

    <script>
        // Chart instances storage
        let charts = {};
        
        // Tab switching function
        function switchTab(tabName) {
            // Hide all tab contents
            document.querySelectorAll('.tab-content').forEach(tab => {
                tab.style.display = 'none';
            });
            
            // Remove active class from all buttons
            document.querySelectorAll('.tab-button').forEach(button => {
                button.classList.remove('active');
            });
            
            // Show selected tab and activate its button
            document.getElementById(tabName).style.display = 'block';
            event.target.classList.add('active');
            
            // Load data for real dashboard if needed
            if (tabName === 'real-dashboard') {
                loadRealTrendsDashboard();
            }
        }
        
        async function predict() {
            const hours = parseFloat(document.getElementById('hours').value);
            const attendance = parseFloat(document.getElementById('attendance').value);
            
            const response = await fetch('/predict', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ hours, attendance })
            });
            
            const result = await response.json();
            const resultDiv = document.getElementById('result');
            
            if (result.prediction === 'Pass') {
                resultDiv.className = 'result pass';
                resultDiv.innerHTML = `
                    <h3>🎉 Prediction Result: PASS</h3>
                    <p><strong>Confidence:</strong> ${(result.confidence * 100).toFixed(1)}%</p>
                    <p><strong>Study Hours:</strong> ${hours} hours/week</p>
                    <p><strong>Attendance:</strong> ${attendance}%</p>
                    <p>✅ Great job! Your current study habits and attendance should lead to success.</p>
                `;
            } else {
                resultDiv.className = 'result fail';
                resultDiv.innerHTML = `
                    <h3>⚠️ Prediction Result: NEEDS IMPROVEMENT</h3>
                    <p><strong>Confidence:</strong> ${(result.confidence * 100).toFixed(1)}%</p>
                    <p><strong>Study Hours:</strong> ${hours} hours/week</p>
                    <p><strong>Attendance:</strong> ${attendance}%</p>
                    <p>💡 Consider increasing study hours and improving attendance for better results.</p>
                `;
            }
            
            resultDiv.style.display = 'block';
        }
        
        async function showAnalytics() {
            const response = await fetch('/analytics');
            const analytics = await response.json();
            
            const content = document.getElementById('analytics-content');
            content.innerHTML = `
                <div class="chart-grid">
                    <div class="metric-card">
                        <h4>Total Students</h4>
                        <div class="metric-value">${analytics.total_students}</div>
                    </div>
                    <div class="metric-card">
                        <h4>Pass Rate</h4>
                        <div class="metric-value">${(analytics.pass_rate * 100).toFixed(1)}%</div>
                    </div>
                    <div class="metric-card">
                        <h4>Avg Study Hours</h4>
                        <div class="metric-value">${analytics.avg_study_hours}</div>
                    </div>
                    <div class="metric-card">
                        <h4>Avg Attendance</h4>
                        <div class="metric-value">${analytics.avg_attendance}%</div>
                    </div>
                </div>
                
                <h4>Performance Breakdown</h4>
                <table class="prediction-table">
                    <thead>
                        <tr>
                            <th>Performance Range</th>
                            <th>Students</th>
                            <th>Pass Rate</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${analytics.performance_breakdown.map(cat => `
                            <tr>
                                <td>${cat.range}</td>
                                <td>${cat.count}</td>
                                <td>${(cat.pass_rate * 100).toFixed(1)}%</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
            
            document.getElementById('analytics').style.display = 'block';
        }
        
        async function showDatabaseAnalytics() {
            const response = await fetch('/database-analytics');
            const analytics = await response.json();
            
            const content = document.getElementById('database-analytics');
            content.className = 'result info';
            content.innerHTML = `
                <h3>💾 Real Database Analytics</h3>
                <div class="chart-grid">
                    <div class="metric-card">
                        <h4>Total Predictions</h4>
                        <div class="metric-value">${analytics.total_students}</div>
                    </div>
                    <div class="metric-card">
                        <h4>Pass Rate</h4>
                        <div class="metric-value">${(analytics.pass_rate * 100).toFixed(1)}%</div>
                    </div>
                    <div class="metric-card">
                        <h4>Avg Study Hours</h4>
                        <div class="metric-value">${analytics.avg_study_hours.toFixed(1)}</div>
                    </div>
                    <div class="metric-card">
                        <h4>Avg Attendance</h4>
                        <div class="metric-value">${analytics.avg_attendance.toFixed(1)}%</div>
                    </div>
                </div>
                <p><em>Based on ${analytics.total_students} predictions stored in the database</em></p>
            `;
            
            content.style.display = 'block';
        }
        
        async function showTips() {
            const response = await fetch('/success-tips');
            const tips = await response.json();
            
            const content = document.getElementById('tips-content');
            content.innerHTML = `
                <ul style="list-style-type: none; padding: 0;">
                    ${tips.map(tip => `<li style="padding: 10px; margin: 10px 0; background: white; border-radius: 8px; border-left: 4px solid #17a2b8;">${tip}</li>`).join('')}
                </ul>
            `;
            
            document.getElementById('tips').style.display = 'block';
        }
        
        async function showModelInfo() {
            const response = await fetch('/model-info');
            const modelInfo = await response.json();
            
            const content = document.getElementById('model-info-content');
            content.innerHTML = `
                <div class="metric-card">
                    <h4>Model Accuracy</h4>
                    <div class="metric-value">${(modelInfo.accuracy * 100).toFixed(1)}%</div>
                </div>
                <p><strong>Features Used:</strong> ${modelInfo.features.join(', ')}</p>
                <p><strong>Training Data Size:</strong> ${modelInfo.training_data_size} samples</p>
                <p><strong>Model Type:</strong> ${modelInfo.model_type}</p>
                <p><strong>Last Updated:</strong> ${new Date(modelInfo.last_updated).toLocaleDateString()}</p>
            `;
            
            document.getElementById('model-info').style.display = 'block';
        }
        
        async function processBatch() {
            const batchData = document.getElementById('batchData').value;
            const lines = batchData.trim().split('\n');
            const headers = lines[0].split(',').map(h => h.trim());
            const students = [];
            
            for (let i = 1; i < lines.length; i++) {
                const values = lines[i].split(',').map(v => v.trim());
                if (values.length === headers.length) {
                    const student = {};
                    headers.forEach((header, index) => {
                        if (header === 'hours' || header === 'attendance') {
                            student[header] = parseFloat(values[index]);
                        } else {
                            student[header] = values[index];
                        }
                    });
                    students.push(student);
                }
            }
            
            const response = await fetch('/batch-predict', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(students)
            });
            
            const result = await response.json();
            const resultDiv = document.getElementById('batch-result');
            
            let tableHtml = `
                <h3>📊 Batch Prediction Results</h3>
                <p><strong>Total Processed:</strong> ${result.predictions.length} students</p>
                <table class="prediction-table">
                    <thead>
                        <tr>
                            <th>Student Name</th>
                            <th>Study Hours</th>
                            <th>Attendance</th>
                            <th>Prediction</th>
                            <th>Confidence</th>
                        </tr>
                    </thead>
                    <tbody>
            `;
            
            result.predictions.forEach(student => {
                tableHtml += `
                    <tr>
                        <td>${student.name}</td>
                        <td>${student.hours}</td>
                        <td>${student.attendance}%</td>
                        <td>${student.prediction}</td>
                        <td>${(student.confidence * 100).toFixed(1)}%</td>
                    </tr>
                `;
            });
            
            tableHtml += '</tbody></table>';
            resultDiv.innerHTML = tableHtml;
            resultDiv.className = 'result success';
            resultDiv.style.display = 'block';
        }
        
        async function loadTrendsDashboard() {
            const response = await fetch('/trends-dashboard');
            const dashboard = await response.json();
            
            const resultDiv = document.getElementById('trends-dashboard');
            resultDiv.innerHTML = `
                <h3>📈 Interactive Trends Dashboard</h3>
                <p><strong>Generated:</strong> ${new Date(dashboard.timestamp).toLocaleString()}</p>
                
                <div class="chart-container">
                    <canvas id="classTrendsChart"></canvas>
                </div>
                
                <h4>Student Trends</h4>
                ${dashboard.student_trends.map(trend => `
                    <div class="trend-card ${trend.trend.toLowerCase()}">
                        <h5>${trend.student_name}</h5>
                        <p><strong>Trend:</strong> ${trend.trend}</p>
                        <p><strong>Current Performance:</strong> ${trend.current_performance}</p>
                        <p><strong>Recommendation:</strong> ${trend.recommendation}</p>
                    </div>
                `).join('')}
            `;
            
            resultDiv.className = 'result info';
            resultDiv.style.display = 'block';
            
            // Initialize chart
            const ctx = document.getElementById('classTrendsChart').getContext('2d');
            if (charts.classTrends) {
                charts.classTrends.destroy();
            }
            
            charts.classTrends = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: dashboard.class_trends.weeks,
                    datasets: [
                        {
                            label: 'Study Hours',
                            data: dashboard.class_trends.avg_study_hours,
                            borderColor: '#007bff',
                            backgroundColor: 'rgba(0,123,255,0.1)',
                            tension: 0.4
                        },
                        {
                            label: 'Attendance %',
                            data: dashboard.class_trends.avg_attendance,
                            borderColor: '#28a745',
                            backgroundColor: 'rgba(40,167,69,0.1)',
                            tension: 0.4
                        }
                    ]
                },
                options: {
                    responsive: true,
                    plugins: {
                        title: {
                            display: true,
                            text: 'Class Performance Trends Over Time'
                        }
                    },
                    scales: {
                        y: {
                            beginAtZero: true
                        }
                    }
                }
            });
        }
        
        async function loadRealTrendsDashboard() {
            const response = await fetch('/real-trends-dashboard');
            const dashboard = await response.json();
            
            const resultDiv = document.getElementById('real-trends-dashboard');
            resultDiv.innerHTML = `
                <h3>📈 Real Database Trends Dashboard</h3>
                <p><strong>Data Source:</strong> ${dashboard.data_source}</p>
                <p><strong>Total Records:</strong> ${dashboard.total_records}</p>
                <p><strong>Generated:</strong> ${new Date(dashboard.timestamp).toLocaleString()}</p>
                
                <div class="chart-container">
                    <canvas id="realTrendsChart"></canvas>
                </div>
                
                <h4>Weekly Trends from Database</h4>
                <table class="prediction-table">
                    <thead>
                        <tr>
                            <th>Week</th>
                            <th>Avg Study Hours</th>
                            <th>Avg Attendance</th>
                            <th>Pass Rate</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${dashboard.weekly_trends.map(trend => `
                            <tr>
                                <td>${trend.week}</td>
                                <td>${trend.avg_study_hours.toFixed(1)}</td>
                                <td>${trend.avg_attendance.toFixed(1)}%</td>
                                <td>${(trend.pass_rate * 100).toFixed(1)}%</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
            
            resultDiv.className = 'result info';
            resultDiv.style.display = 'block';
            
            // Initialize real trends chart
            const ctx = document.getElementById('realTrendsChart').getContext('2d');
            if (charts.realTrends) {
                charts.realTrends.destroy();
            }
            
            charts.realTrends = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: dashboard.weekly_trends.map(t => `Week ${t.week}`),
                    datasets: [
                        {
                            label: 'Study Hours',
                            data: dashboard.weekly_trends.map(t => t.avg_study_hours),
                            borderColor: '#007bff',
                            backgroundColor: 'rgba(0,123,255,0.1)',
                            tension: 0.4
                        },
                        {
                            label: 'Attendance %',
                            data: dashboard.weekly_trends.map(t => t.avg_attendance),
                            borderColor: '#28a745',
                            backgroundColor: 'rgba(40,167,69,0.1)',
                            tension: 0.4
                        }
                    ]
                },
                options: {
                    responsive: true,
                    plugins: {
                        title: {
                            display: true,
                            text: 'Real Database Performance Trends'
                        }
                    },
                    scales: {
                        y: {
                            beginAtZero: true
                        }
                    }
                }
            });
        }
        
        async function showAllPredictions() {
            const response = await fetch('/all-predictions');
            const predictions = await response.json();
            
            const content = document.getElementById('all-predictions');
            content.className = 'result info';
            
            let tableHtml = `
                <h3>🗃️ All Predictions from Database</h3>
                <p><strong>Total Records:</strong> ${predictions.length}</p>
                <table class="prediction-table">
                    <thead>
                        <tr>
                            <th>Student Name</th>
                            <th>Study Hours</th>
                            <th>Attendance</th>
                            <th>Prediction</th>
                            <th>Confidence</th>
                            <th>Date</th>
                        </tr>
                    </thead>
                    <tbody>
            `;
            
            predictions.forEach(pred => {
                tableHtml += `
                    <tr>
                        <td>${pred.name}</td>
                        <td>${pred.study_hours}</td>
                        <td>${pred.attendance}%</td>
                        <td>${pred.predicted_pass ? 'Pass' : 'Fail'}</td>
                        <td>${(pred.confidence * 100).toFixed(1)}%</td>
                        <td>${new Date(pred.created_at).toLocaleDateString()}</td>
                    </tr>
                `;
            });
            
            tableHtml += '</tbody></table>';
            content.innerHTML = tableHtml;
            content.style.display = 'block';
        }
        
        async function trackProgress() {
            const studentName = document.getElementById('studentName').value;
            const weeks = parseInt(document.getElementById('trackingWeeks').value);
            
            const response = await fetch('/track-progress', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ student_name: studentName, weeks })
            });
            
            const progress = await response.json();
            const resultDiv = document.getElementById('progress-result');
            
            let progressHtml = `
                <h3>📈 Progress Tracking: ${progress.student_name}</h3>
                <p><strong>Tracking Period:</strong> ${progress.tracking_weeks} weeks</p>
                <p><strong>Overall Trend:</strong> <span class="${progress.overall_trend.toLowerCase()}">${progress.overall_trend}</span></p>
                <p><strong>Generated:</strong> ${new Date(progress.generated_at).toLocaleString()}</p>
                
                <div class="chart-container">
                    <canvas id="progressChart"></canvas>
                </div>
                
                <h4>Weekly Progress Data</h4>
                <table class="prediction-table">
                    <thead>
                        <tr>
                            <th>Week</th>
                            <th>Study Hours</th>
                            <th>Attendance</th>
                            <th>Prediction</th>
                            <th>Confidence</th>
                            <th>Improvement Score</th>
                        </tr>
                    </thead>
                    <tbody>
            `;
            
            progress.progress_data.forEach(week => {
                progressHtml += `
                    <tr>
                        <td>${week.week}</td>
                        <td>${week.study_hours.toFixed(1)}</td>
                        <td>${week.attendance.toFixed(1)}%</td>
                        <td>${week.prediction}</td>
                        <td>${(week.confidence * 100).toFixed(1)}%</td>
                        <td>${week.improvement_score.toFixed(1)}/10</td>
                    </tr>
                `;
            });
            
            progressHtml += '</tbody></table>';
            resultDiv.innerHTML = progressHtml;
            resultDiv.className = 'result info';
            resultDiv.style.display = 'block';
            
            // Initialize progress chart
            const ctx = document.getElementById('progressChart').getContext('2d');
            if (charts.progressChart) {
                charts.progressChart.destroy();
            }
            
            charts.progressChart = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: progress.progress_data.map(w => `Week ${w.week}`),
                    datasets: [
                        {
                            label: 'Study Hours',
                            data: progress.progress_data.map(w => w.study_hours),
                            borderColor: '#007bff',
                            backgroundColor: 'rgba(0,123,255,0.1)',
                            yAxisID: 'y'
                        },
                        {
                            label: 'Attendance %',
                            data: progress.progress_data.map(w => w.attendance),
                            borderColor: '#28a745',
                            backgroundColor: 'rgba(40,167,69,0.1)',
                            yAxisID: 'y'
                        },
                        {
                            label: 'Improvement Score',
                            data: progress.progress_data.map(w => w.improvement_score),
                            borderColor: '#ffc107',
                            backgroundColor: 'rgba(255,193,7,0.1)',
                            yAxisID: 'y1'
                        }
                    ]
                },
                options: {
                    responsive: true,
                    interaction: {
                        mode: 'index',
                        intersect: false,
                    },
                    scales: {
                        y: {
                            type: 'linear',
                            display: true,
                            position: 'left',
                        },
                        y1: {
                            type: 'linear',
                            display: true,
                            position: 'right',
                            max: 10,
                            grid: {
                                drawOnChartArea: false,
                            },
                        },
                    }
                }
            });
        }
        
        async function generateStudyPlan() {
            const studentName = document.getElementById('planStudentName').value;
            const currentHours = parseFloat(document.getElementById('currentHours').value);
            const currentAttendance = parseFloat(document.getElementById('currentAttendance').value);
            const targetGrade = document.getElementById('targetGrade').value;
            
            // Get selected days
            const days = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];
            const availableDays = days.filter(day => document.getElementById(day).checked);
            
            // Get selected times
            const times = ['morning', 'afternoon', 'evening'];
            const preferredTimes = times.filter(time => document.getElementById(time).checked);
            
            const request = {
                student_name: studentName,
                current_hours: currentHours,
                current_attendance: currentAttendance,
                target_grade: targetGrade,
                available_days: availableDays,
                preferred_times: preferredTimes
            };
            
            const response = await fetch('/generate-study-plan', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(request)
            });
            
            const studyPlan = await response.json();
            const resultDiv = document.getElementById('study-plan-result');
            
            let planHtml = `
                <h3>📚 Personalized Study Plan for ${studyPlan.student_name}</h3>
                <p><strong>Target Grade:</strong> ${studyPlan.target_grade}</p>
                <p><strong>Recommended Weekly Hours:</strong> ${studyPlan.recommended_hours}</p>
                <p><strong>Target Attendance:</strong> ${studyPlan.target_attendance}%</p>
                <p><strong>Plan Duration:</strong> ${studyPlan.plan_duration} weeks</p>
                
                <h4>📅 Weekly Study Schedule</h4>
            `;
            
            studyPlan.weekly_schedule.forEach(day => {
                planHtml += `
                    <div class="schedule-day">
                        <h5>${day.day}</h5>
                        ${day.study_blocks.map(block => `
                            <div class="study-block">
                                <strong>${block.time}</strong>: ${block.subject} - ${block.activity}
                                <br><small>Duration: ${block.duration} hours</small>
                            </div>
                        `).join('')}
                    </div>
                `;
            });
            
            planHtml += `
                <h4>💡 Study Recommendations</h4>
                ${studyPlan.recommendations.map(rec => `
                    <div class="recommendation-item">
                        ${rec}
                    </div>
                `).join('')}
                
                <h4>🎯 Expected Outcomes</h4>
                <p>${studyPlan.expected_outcomes}</p>
                
                <p><em>Generated on: ${new Date(studyPlan.generated_at).toLocaleString()}</em></p>
            `;
            
            resultDiv.innerHTML = planHtml;
            resultDiv.className = 'result success';
            resultDiv.style.display = 'block';
        }

        // Gamification functions
        async function recordStudySession() {
            const studentName = document.getElementById('sessionStudentName').value;
            const duration = parseFloat(document.getElementById('sessionDuration').value);
            const subjects = document.getElementById('sessionSubjects').value.split(',').map(s => s.trim());
            const focusScore = parseInt(document.getElementById('focusScore').value) / 100.0;
            const attendanceToday = document.getElementById('attendanceToday').checked;

            const request = {
                student_name: studentName,
                duration_hours: duration,
                subjects: subjects,
                focus_score: focusScore,
                attendance_today: attendanceToday
            };

            const response = await fetch('/record-session', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(request)
            });

            const result = await response.json();
            const resultDiv = document.getElementById('gamification-result');
            
            let html = `
                <h3>🎉 Study Session Recorded!</h3>
                <div class="metric-card">
                    <h4>Points Earned</h4>
                    <div class="metric-value" style="color: #ff8c00;">+${result.points_earned}</div>
                </div>
                
                <p><strong>Total Points:</strong> ${result.profile.total_points + result.points_earned}</p>
                <p><strong>Level:</strong> ${result.profile.level} ${result.level_up ? '🎊 LEVEL UP!' : ''}</p>
                <p><strong>Current Streak:</strong> ${result.profile.current_streak} days 🔥</p>
                <p><strong>Leaderboard Position:</strong> #${result.leaderboard_position}</p>
            `;

            if (result.new_badges.length > 0) {
                html += `<h4>🏅 New Badges Earned:</h4>`;
                result.new_badges.forEach(badge => {
                    html += `
                        <div class="badge-item">
                            <strong>${badge.icon} ${badge.name}</strong> - ${badge.description}
                            <br><small>Rarity: ${badge.rarity}</small>
                        </div>
                    `;
                });
            }

            if (result.new_achievements.length > 0) {
                html += `<h4>⭐ New Achievements Unlocked:</h4>`;
                result.new_achievements.forEach(achievement => {
                    html += `
                        <div class="achievement-item">
                            <strong>${achievement.name}</strong> - ${achievement.description}
                            <br><small>+${achievement.points} points</small>
                        </div>
                    `;
                });
            }

            resultDiv.innerHTML = html;
            resultDiv.className = 'result success';
            resultDiv.style.display = 'block';
        }

        async function showLeaderboard() {
            const response = await fetch('/leaderboard');
            const leaderboard = await response.json();
            
            const resultDiv = document.getElementById('leaderboard-result');
            
            let tableHtml = `
                <h3>🏆 Study Leaderboard</h3>
                <table class="prediction-table">
                    <thead>
                        <tr>
                            <th>Rank</th>
                            <th>Student</th>
                            <th>Points</th>
                            <th>Level</th>
                            <th>Badges</th>
                        </tr>
                    </thead>
                    <tbody>
            `;
            
            leaderboard.forEach(entry => {
                const rankClass = entry.rank <= 3 ? `style="background: ${entry.rank === 1 ? '#ffd700' : entry.rank === 2 ? '#c0c0c0' : '#cd7f32'};"` : '';
                tableHtml += `
                    <tr ${rankClass}>
                        <td>${entry.rank}</td>
                        <td>${entry.student_name}</td>
                        <td>${entry.total_points}</td>
                        <td>${entry.level}</td>
                        <td>${entry.badge_count} 🏅</td>
                    </tr>
                `;
            });
            
            tableHtml += '</tbody></table>';
            resultDiv.innerHTML = tableHtml;
            resultDiv.className = 'result info';
            resultDiv.style.display = 'block';
        }

        async function showMyProfile() {
            const studentName = document.getElementById('sessionStudentName').value;
            
            const response = await fetch('/profile/' + encodeURIComponent(studentName));
            const profile = await response.json();
            
            const resultDiv = document.getElementById('profile-result');
            
            let html = `
                <h3>👤 Student Profile: ${profile.student_name}</h3>
                <div class="chart-grid">
                    <div class="metric-card">
                        <h4>Total Points</h4>
                        <div class="metric-value" style="color: #ff8c00;">${profile.total_points}</div>
                    </div>
                    <div class="metric-card">
                        <h4>Level</h4>
                        <div class="metric-value" style="color: #28a745;">${profile.level}</div>
                    </div>
                    <div class="metric-card">
                        <h4>Current Streak</h4>
                        <div class="metric-value" style="color: #dc3545;">${profile.current_streak} days</div>
                    </div>
                    <div class="metric-card">
                        <h4>Badges</h4>
                        <div class="metric-value" style="color: #6f42c1;">${profile.badges.length}</div>
                    </div>
                </div>
                
                <h4>🏅 Badges Earned</h4>
                <div style="display: flex; flex-wrap: wrap; gap: 10px;">
            `;
            
            profile.badges.forEach(badge => {
                html += `
                    <div style="background: white; padding: 15px; border-radius: 10px; text-align: center; min-width: 120px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                        <div style="font-size: 2em;">${badge.icon}</div>
                        <strong>${badge.name}</strong>
                        <br><small style="color: #666;">${badge.rarity}</small>
                    </div>
                `;
            });
            
            html += `</div>`;
            
            resultDiv.innerHTML = html;
            resultDiv.className = 'result info';
            resultDiv.style.display = 'block';
        }
        
        // Initialize the first tab as active
        document.getElementById('mock-dashboard').style.display = 'block';
    </script>
</body>
</html>"#;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the ML model
    let model_info = train_model();
    let trained_model = TrainedModel::new();
    
    // Initialize database
    let database = Database::new().await
        .expect("Failed to initialize database");
    
    // Initialize gamification engine
    let gamification_engine = GamificationEngine::new();
    
    // Create application data
    let model_data = web::Data::new(model_info);
    let trained_model_data = web::Data::new(trained_model);
    let db_data = web::Data::new(database);
    let gamification_data = web::Data::new(gamification_engine);
    
    println!("🚀 Starting TUK Student Classifier Server at http://localhost:8080");
    println!("📊 Student Performance Analytics Dashboard ready!");
    println!("🎓 Study Plan Generator feature activated!");
    println!("🎮 Gamification System activated!");
    
    HttpServer::new(move || {
        App::new()
            .app_data(model_data.clone())
            .app_data(trained_model_data.clone())
            .app_data(db_data.clone())
            .app_data(gamification_data.clone())
            .route("/", web::get().to(serve_homepage))
            .route("/predict", web::post().to(predict))
            .route("/batch-predict", web::post().to(batch_predict))
            .route("/analytics", web::get().to(get_analytics))
            .route("/database-analytics", web::get().to(get_database_analytics))
            .route("/success-tips", web::get().to(get_success_tips))
            .route("/model-info", web::get().to(get_model_info))
            .route("/health", web::get().to(health_check))
            .route("/student-trends", web::post().to(get_student_trends))
            .route("/class-trends", web::get().to(get_class_trends))
            .route("/trends-dashboard", web::get().to(get_trends_dashboard))
            .route("/real-trends-dashboard", web::get().to(get_real_trends_dashboard))
            .route("/all-predictions", web::get().to(get_all_predictions))
            .route("/save-model-version", web::post().to(save_model_version))
            .route("/track-progress", web::post().to(track_student_progress))
            .route("/generate-study-plan", web::post().to(generate_study_plan))
            // NEW: Gamification endpoints
            .route("/record-session", web::post().to(record_study_session))
            .route("/leaderboard", web::get().to(get_leaderboard))
            .route("/profile/{student_name}", web::get().to(get_student_profile))
            .route("/achievements/{student_name}", web::get().to(get_achievements))
            .route("/badges/{student_name}", web::get().to(get_badges))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}