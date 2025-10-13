mod data;
mod model;
mod analytics;
mod database;

use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::error::Error;
use rand::Rng;

use crate::model::{train_model, ModelInfo, PredictResponse, AnalyticsData, PerformanceCategory, 
                   StudentRecord as ModelStudentRecord, TrainedModel};
use crate::analytics::{TrendsAnalyzer, generate_mock_trends_data};
use crate::database::{Database, StudentRecord as DbStudentRecord, ModelVersion};

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

// NEW: Student progress tracking request
#[derive(Deserialize)]
struct ProgressRequest {
    student_name: String,
    weeks: usize,
}

// Student trends endpoint
async fn get_student_trends(
    web::Json(req): web::Json<StudentTrendsRequest>,
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

// NEW: Student progress tracking endpoint
async fn track_student_progress(
    web::Json(req): web::Json<ProgressRequest>,
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
    web::Json(students): web::Json<Vec<ModelStudentRecord>>,
    model: web::Data<TrainedModel>,
    db: web::Data<Database>,
) -> HttpResponse {
    let batch_result = model.batch_predict(students);
    
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

// Homepage endpoint with complete HTML
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

        input {
            width: 100%;
            padding: 15px;
            border: 2px solid #e9ecef;
            border-radius: 10px;
            font-size: 16px;
            transition: all 0.3s ease;
            background: white;
        }

        input:focus {
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

        <!-- NEW: Student Progress Tracking Section -->
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
            
            // Show selected tab and activate button
            document.getElementById(tabName).style.display = 'block';
            event.target.classList.add('active');
        }

        // Show initial tab
        switchTab('mock-dashboard');

        // Prediction function
        async function predict() {
            const hours = document.getElementById('hours').value;
            const attendance = document.getElementById('attendance').value;
            const resultDiv = document.getElementById('result');
            
            if (!hours || !attendance) {
                resultDiv.style.display = 'block';
                resultDiv.className = 'result warning';
                resultDiv.innerHTML = '<p>⚠️ Please enter both study hours and attendance</p>';
                return;
            }
            
            try {
                const response = await fetch('/predict', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({hours: parseFloat(hours), attendance: parseFloat(attendance)})
                });
                
                const data = await response.json();
                
                resultDiv.style.display = 'block';
                resultDiv.className = 'result ' + (data.prediction === 'Pass' ? 'pass' : 'fail');
                resultDiv.innerHTML = `
                    <h3>🎓 Prediction Result: ${data.prediction}</h3>
                    <p><strong>Confidence Level:</strong> ${(data.confidence * 100).toFixed(1)}%</p>
                    <p>Student with <strong>${hours} study hours</strong> and <strong>${attendance}% attendance</strong> is predicted to: <strong>${data.prediction}</strong></p>
                    <p style="margin-top: 15px; font-size: 0.9em; opacity: 0.8;">✅ This prediction has been saved to the database</p>
                `;
            } catch (error) {
                resultDiv.style.display = 'block';
                resultDiv.className = 'result fail';
                resultDiv.innerHTML = `<p>❌ Error: ${error.message}</p>`;
            }
        }

        // Batch prediction function
        async function processBatch() {
            const batchData = document.getElementById('batchData').value;
            const resultDiv = document.getElementById('batch-result');
            
            try {
                const lines = batchData.trim().split('\n');
                const students = [];
                
                for (let i = 1; i < lines.length; i++) {
                    if (lines[i].trim()) {
                        const values = lines[i].split(',').map(v => v.trim());
                        if (values.length >= 3) {
                            students.push({
                                name: values[0],
                                hours: parseFloat(values[1]),
                                attendance: parseFloat(values[2])
                            });
                        }
                    }
                }
                
                if (students.length === 0) {
                    throw new Error('No valid student data found. Please check CSV format.');
                }
                
                const response = await fetch('/batch-predict', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify(students)
                });
                
                const data = await response.json();
                
                resultDiv.style.display = 'block';
                resultDiv.className = 'result info';
                resultDiv.innerHTML = `
                    <h3>📊 Batch Prediction Results</h3>
                    <div style="background: white; padding: 15px; border-radius: 5px; margin: 15px 0;">
                        <h4>Summary</h4>
                        <p><strong>Total Students:</strong> ${data.total_students}</p>
                        <p><strong>Pass Rate:</strong> ${(data.summary.pass_rate * 100).toFixed(1)}%</p>
                        <p><strong>Pass Count:</strong> ${data.summary.pass_count} | <strong>Fail Count:</strong> ${data.summary.fail_count}</p>
                        <p><strong>Average Confidence:</strong> ${(data.summary.avg_confidence * 100).toFixed(1)}%</p>
                        <p><small>✅ All predictions have been saved to the database</small></p>
                    </div>
                    
                    <h4>Individual Predictions</h4>
                    <table class="prediction-table">
                        <thead>
                            <tr>
                                <th>Student</th>
                                <th>Hours</th>
                                <th>Attendance</th>
                                <th>Prediction</th>
                                <th>Confidence</th>
                                <th>Recommendation</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${data.predictions.map(pred => `
                                <tr>
                                    <td>${pred.name}</td>
                                    <td>${pred.hours}</td>
                                    <td>${pred.attendance}%</td>
                                    <td style="color: ${pred.prediction === 'Pass' ? '#28a745' : '#dc3545'}; font-weight: bold;">
                                        ${pred.prediction}
                                    </td>
                                    <td>${(pred.confidence * 100).toFixed(1)}%</td>
                                    <td>${pred.recommendation}</td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>
                `;
                
                resultDiv.scrollIntoView({behavior: 'smooth'});
            } catch (error) {
                resultDiv.style.display = 'block';
                resultDiv.className = 'result fail';
                resultDiv.innerHTML = `<p>❌ Error processing batch: ${error.message}</p>`;
            }
        }

        // NEW: Student progress tracking function
        async function trackProgress() {
            const studentName = document.getElementById('studentName').value;
            const trackingWeeks = document.getElementById('trackingWeeks').value;
            const resultDiv = document.getElementById('progress-result');
            
            if (!studentName || !trackingWeeks) {
                resultDiv.style.display = 'block';
                resultDiv.className = 'result warning';
                resultDiv.innerHTML = '<p>⚠️ Please enter student name and weeks to track</p>';
                return;
            }
            
            try {
                const response = await fetch('/track-progress', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({
                        student_name: studentName,
                        weeks: parseInt(trackingWeeks)
                    })
                });
                
                const data = await response.json();
                
                resultDiv.style.display = 'block';
                resultDiv.className = 'result info';
                resultDiv.innerHTML = `
                    <h3>📈 Progress Report: ${data.student_name}</h3>
                    <p><strong>Tracking Period:</strong> ${data.tracking_weeks} weeks</p>
                    <p><strong>Overall Trend:</strong> <span style="color: ${
                        data.overall_trend === 'Improving' ? '#28a745' : 
                        data.overall_trend === 'Declining' ? '#dc3545' : '#ffc107'
                    };">${data.overall_trend}</span></p>
                    
                    <h4>Weekly Progress</h4>
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
                            ${data.progress_data.map(week => `
                                <tr>
                                    <td>${week.week}</td>
                                    <td>${week.study_hours.toFixed(1)}h</td>
                                    <td>${week.attendance.toFixed(1)}%</td>
                                    <td style="color: ${week.prediction === 'Pass' ? '#28a745' : '#dc3545'}; font-weight: bold;">
                                        ${week.prediction}
                                    </td>
                                    <td>${(week.confidence * 100).toFixed(1)}%</td>
                                    <td>
                                        <div style="background: #e9ecef; border-radius: 10px; height: 10px; width: 100px; margin: 5px 0;">
                                            <div style="background: linear-gradient(90deg, #dc3545, #ffc107, #28a745); 
                                                        width: ${week.improvement_score * 10}%; 
                                                        height: 100%; 
                                                        border-radius: 10px;"></div>
                                        </div>
                                        ${week.improvement_score.toFixed(1)}/10
                                    </td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>
                    
                    <div style="margin-top: 20px; padding: 15px; background: white; border-radius: 8px;">
                        <h5>📋 Progress Summary</h5>
                        <p><strong>Generated:</strong> ${new Date(data.generated_at).toLocaleString()}</p>
                        <p><strong>Recommendation:</strong> ${
                            data.overall_trend === 'Improving' ? 
                            'Great progress! Continue with current study habits.' :
                            data.overall_trend === 'Declining' ?
                            'Consider adjusting study strategies and seeking academic support.' :
                            'Maintain consistency and look for areas of improvement.'
                        }</p>
                    </div>
                `;
                
                resultDiv.scrollIntoView({behavior: 'smooth'});
            } catch (error) {
                resultDiv.style.display = 'block';
                resultDiv.className = 'result fail';
                resultDiv.innerHTML = `<p>❌ Error tracking progress: ${error.message}</p>`;
            }
        }

        // Placeholder functions for other features
        async function showAnalytics() {
            alert('Analytics feature would be implemented here');
        }

        async function showDatabaseAnalytics() {
            alert('Database Analytics feature would be implemented here');
        }

        async function showTips() {
            alert('Success Tips feature would be implemented here');
        }

        async function showModelInfo() {
            alert('Model Info feature would be implemented here');
        }

        async function loadTrendsDashboard() {
            alert('Trends Dashboard feature would be implemented here');
        }

        async function showAllPredictions() {
            alert('View All Predictions feature would be implemented here');
        }

    </script>
</body>
</html>"#;

    HttpResponse::Ok().content_type("text/html").body(html_content)
}

// Add database to start_api function
async fn start_api(
    model: TrainedModel,
    model_info: ModelInfo,
    db: Database,
) -> std::io::Result<()> {
    let model_data = web::Data::new(model);
    let info_data = web::Data::new(model_info);
    let db_data = web::Data::new(db);
    
    HttpServer::new(move || {
        App::new()
            .app_data(model_data.clone())
            .app_data(info_data.clone())
            .app_data(db_data.clone())
            .route("/", web::get().to(serve_homepage))
            .route("/predict", web::post().to(predict))
            .route("/batch-predict", web::post().to(batch_predict))
            .route("/model/info", web::get().to(get_model_info))
            .route("/health", web::get().to(health_check))
            .route("/analytics", web::get().to(get_analytics))
            .route("/database-analytics", web::get().to(get_database_analytics))
            .route("/real-trends-dashboard", web::get().to(get_real_trends_dashboard))
            .route("/all-predictions", web::get().to(get_all_predictions))
            .route("/save-model-version", web::post().to(save_model_version))
            .route("/tips", web::get().to(get_success_tips))
            .route("/student-trends", web::post().to(get_student_trends))
            .route("/class-trends", web::get().to(get_class_trends))
            .route("/trends-dashboard", web::get().to(get_trends_dashboard))
            .route("/track-progress", web::post().to(track_student_progress))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting TUK Student Classifier...");
    
    // Initialize database
    println!("🗃️ Initializing database...");
    let db = Database::new().await?;
    println!("✅ Database initialized successfully!");

    // Train model (simple rule-based)
    println!("🤖 Training student performance model...");
    let (model, accuracy) = train_model()?;
    println!("🎯 Model trained successfully! Accuracy: {:.2}%", accuracy * 100.0);

    let model_info = ModelInfo { accuracy };

    // Save model version to database
    let model_version = ModelVersion {
        id: 0,
        version: "1.0.0".to_string(),
        accuracy,
        features_used: "study_hours,attendance".to_string(),
        created_at: chrono::Utc::now(),
    };
    
    if let Err(e) = db.save_model_version(&model_version).await {
        eprintln!("Warning: Failed to save model version: {}", e);
    } else {
        println!("✅ Model version saved to database");
    }

    println!("🌐 Starting TUK Student Predictor API on http://127.0.0.1:8080");
    println!("   Visit http://127.0.0.1:8080 in your browser!");
    println!("   Features: Real Database Analytics & Rule-based Predictions");
    
    start_api(model, model_info, db).await?;

    Ok(())
}