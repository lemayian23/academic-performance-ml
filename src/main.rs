mod data;
mod model;
mod analytics;

use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::error::Error;

use crate::data::load_data;
use crate::model::{train_model, ModelInfo, PredictResponse, AnalyticsData, PerformanceCategory, 
                   StudentRecord, TrainedModel};
use crate::analytics::{TrendsAnalyzer, StudentTrend, ClassTrends, generate_mock_trends_data};

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

// ... (KEEP ALL YOUR EXISTING ENDPOINTS - they remain the same)
// Prediction endpoint
async fn predict(
    req: web::Json<PredictRequest>,
    model: web::Data<TrainedModel>,
) -> HttpResponse {
    let features = vec![req.hours, req.attendance];
    let (prediction, confidence) = model.predict(&features);
    
    let response = PredictResponse {
        prediction: if prediction { "Pass".to_string() } else { "Fail".to_string() },
        confidence,
    };
    
    HttpResponse::Ok().json(response)
}

// Batch prediction endpoint
async fn batch_predict(
    web::Json(students): web::Json<Vec<StudentRecord>>,
    model: web::Data<TrainedModel>,
) -> HttpResponse {
    let batch_result = model.batch_predict(students);
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

// Homepage endpoint - UPDATED WITH CHART.JS
async fn serve_homepage() -> HttpResponse {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>The Technical University of Kenya - Student Performance Predictor</title>
        <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
        <style>
            body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; background: #f8f9fa; }
            .container { background: white; padding: 30px; border-radius: 15px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
            .header { text-align: center; margin-bottom: 30px; border-bottom: 3px solid #007bff; padding-bottom: 20px; }
            .form-group { margin: 20px 0; }
            label { display: block; margin-bottom: 8px; font-weight: 600; color: #495057; }
            input, textarea { width: 100%; padding: 12px; border: 2px solid #e9ecef; border-radius: 8px; font-size: 16px; transition: border-color 0.3s; }
            input:focus, textarea:focus { border-color: #007bff; outline: none; }
            button { background: #007bff; color: white; padding: 14px 28px; border: none; border-radius: 8px; cursor: pointer; margin: 8px; font-size: 16px; font-weight: 600; transition: all 0.3s; }
            button:hover { background: #0056b3; transform: translateY(-2px); box-shadow: 0 4px 8px rgba(0,0,0,0.2); }
            .result { margin-top: 25px; padding: 25px; border-radius: 10px; display: none; border-left: 5px solid; }
            .pass { background: #d4edda; color: #155724; border-left-color: #28a745; }
            .fail { background: #f8d7da; color: #721c24; border-left-color: #dc3545; }
            .warning { background: #fff3cd; color: #856404; border-left-color: #ffc107; }
            .info { background: #d1ecf1; color: #0c5460; border-left-color: #17a2b8; }
            .button-group { text-align: center; margin: 30px 0; display: flex; justify-content: center; flex-wrap: wrap; gap: 10px; }
            .feature-section { background: #e8f5e8; padding: 25px; border-radius: 12px; margin: 25px 0; border: 2px solid #28a745; }
            .trends-section { background: #fff3e6; padding: 25px; border-radius: 12px; margin: 25px 0; border: 2px solid #fd7e14; }
            .prediction-table { width: 100%; border-collapse: collapse; margin: 20px 0; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
            .prediction-table th { background: #343a40; color: white; padding: 15px; text-align: left; }
            .prediction-table td { padding: 12px; border-bottom: 1px solid #dee2e6; }
            .prediction-table tr:hover { background: #f8f9fa; }
            .chart-container { background: white; padding: 20px; border-radius: 10px; margin: 20px 0; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
            .chart-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0; }
            @media (max-width: 768px) { .chart-grid { grid-template-columns: 1fr; } }
            .trend-card { background: white; padding: 20px; margin: 15px 0; border-radius: 10px; border-left: 5px solid; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
            .improving { border-left-color: #28a745; }
            .declining { border-left-color: #dc3545; }
            .stable { border-left-color: #ffc107; }
            .metric-card { background: white; padding: 20px; border-radius: 10px; text-align: center; box-shadow: 0 2px 4px rgba(0,0,0,0.1); border-top: 4px solid; }
            .metric-value { font-size: 2.5em; font-weight: bold; margin: 10px 0; }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1 style="color: #007bff; margin: 0;">🎓 The Technical University of Kenya</h1>
                <h2 style="color: #495057; margin: 10px 0 0 0;">Student Performance Analytics Dashboard</h2>
            </div>
            
            <div class="form-group">
                <label for="hours">📚 Weekly Study Hours:</label>
                <input type="number" id="hours" step="0.1" placeholder="e.g., 6.0" value="6.0">
            </div>
            
            <div class="form-group">
                <label for="attendance">🏫 Class Attendance (%):</label>
                <input type="number" id="attendance" step="0.1" placeholder="e.g., 85.0" value="85.0">
            </div>
            
            <div style="text-align: center;">
                <button onclick="predict()" style="background: linear-gradient(135deg, #007bff, #0056b3);">📊 Predict Academic Result</button>
            </div>
            
            <div id="result" class="result"></div>

            <div class="button-group">
                <button onclick="showAnalytics()" style="background: #28a745;">📈 Performance Analytics</button>
                <button onclick="showTips()" style="background: #6f42c1;">💡 Success Tips</button>
                <button onclick="showModelInfo()" style="background: #fd7e14;">🤖 Model Info</button>
                <button onclick="loadTrendsDashboard()" style="background: #e83e8c;">📊 Interactive Dashboard</button>
                <button onclick="loadClassTrends()" style="background: #20c997;">👥 Class Trends</button>
            </div>

            <!-- ENHANCED: Trends Dashboard Section with Charts -->
            <div class="trends-section">
                <h3 style="color: #e83e8c; margin-top: 0;">📈 Advanced Analytics Dashboard</h3>
                <p>Professional-grade performance tracking with interactive visualizations:</p>
                
                <div id="trends-dashboard" class="result" style="display: none;">
                    <!-- Charts will be injected here by JavaScript -->
                </div>
                <div id="class-trends" class="result" style="display: none;"></div>
            </div>

            <!-- Batch Prediction Section -->
            <div class="feature-section">
                <h3 style="color: #28a745; margin-top: 0;">📁 Batch Student Prediction</h3>
                <p>Upload multiple students for bulk analysis:</p>
                
                <textarea id="batchData" placeholder="Enter CSV data:
name,hours,attendance
Denis Lemayian,6.5,85.0
Saitoti Smith,4.0,70.0
Kukutia Johnson,8.0,92.0
Kirionki Williams,3.0,65.0
David Lemoita,7.5,88.0" 
                    rows="8"></textarea>
                
                <div style="text-align: center;">
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

            // ENHANCED: Load Trends Dashboard with Interactive Charts
            async function loadTrendsDashboard() {
                const dashboardDiv = document.getElementById('trends-dashboard');
                const classDiv = document.getElementById('class-trends');
                classDiv.style.display = 'none';
                
                try {
                    dashboardDiv.innerHTML = '<div style="text-align: center; padding: 40px;"><h3>📊 Loading Advanced Analytics...</h3><p>Generating professional visualizations</p></div>';
                    dashboardDiv.style.display = 'block';
                    
                    const response = await fetch('/trends-dashboard');
                    const data = await response.json();
                    
                    // Destroy existing charts
                    Object.values(charts).forEach(chart => chart.destroy());
                    charts = {};
                    
                    dashboardDiv.innerHTML = `
                        <div style="margin-bottom: 30px;">
                            <h3 style="color: #e83e8c; border-bottom: 2px solid #e83e8c; padding-bottom: 10px;">📊 Performance Analytics Dashboard</h3>
                            
                            <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; margin: 20px 0;">
                                <div class="metric-card" style="border-top-color: #007bff;">
                                    <div style="font-size: 0.9em; color: #6c757d;">Total Students</div>
                                    <div class="metric-value" style="color: #007bff;">${data.class_trends.total_students}</div>
                                </div>
                                <div class="metric-card" style="border-top-color: #28a745;">
                                    <div style="font-size: 0.9em; color: #6c757d;">Avg Improvement</div>
                                    <div class="metric-value" style="color: #28a745;">${data.class_trends.average_improvement.toFixed(1)}</div>
                                </div>
                                <div class="metric-card" style="border-top-color: #ffc107;">
                                    <div style="font-size: 0.9em; color: #6c757d;">Top Performers</div>
                                    <div class="metric-value" style="color: #ffc107; font-size: 1.8em;">${data.class_trends.top_performers.length}</div>
                                </div>
                                <div class="metric-card" style="border-top-color: #dc3545;">
                                    <div style="font-size: 0.9em; color: #6c757d;">Need Support</div>
                                    <div class="metric-value" style="color: #dc3545; font-size: 1.8em;">${data.class_trends.at_risk_students.length}</div>
                                </div>
                            </div>
                        </div>

                        <div class="chart-grid">
                            <div class="chart-container">
                                <h4>📈 Class Performance Trends</h4>
                                <canvas id="classPerformanceChart"></canvas>
                            </div>
                            <div class="chart-container">
                                <h4>🎯 Pass Rate Evolution</h4>
                                <canvas id="passRateChart"></canvas>
                            </div>
                        </div>

                        <div class="chart-container">
                            <h4>👥 Student Performance Distribution</h4>
                            <canvas id="studentPerformanceChart"></canvas>
                        </div>

                        <h4 style="margin-top: 30px;">🎓 Individual Student Analytics</h4>
                        ${data.student_trends.map((student, index) => `
                            <div class="trend-card ${student.overall_trend.toLowerCase()}">
                                <div style="display: flex; justify-content: between; align-items: center; margin-bottom: 15px;">
                                    <h5 style="margin: 0; color: #343a40;">${student.student_name}</h5>
                                    <div style="display: flex; gap: 15px;">
                                        <span style="background: ${student.overall_trend === 'Improving' ? '#28a745' : student.overall_trend === 'Declining' ? '#dc3545' : '#ffc107'}; color: white; padding: 5px 10px; border-radius: 15px; font-size: 0.8em;">
                                            ${student.overall_trend}
                                        </span>
                                        <span style="background: #007bff; color: white; padding: 5px 10px; border-radius: 15px; font-size: 0.8em;">
                                            Score: ${student.improvement_score.toFixed(1)}/10
                                        </span>
                                    </div>
                                </div>
                                <div class="chart-container">
                                    <canvas id="studentChart-${index}"></canvas>
                                </div>
                            </div>
                        `).join('')}
                        
                        <div style="margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 5px; text-align: center;">
                            <small>📅 Dashboard generated: ${new Date(data.timestamp).toLocaleString()}</small>
                        </div>
                    `;
                    
                    // Initialize Charts
                    initializeClassPerformanceChart(data.class_trends.chart_data);
                    initializePassRateChart(data.class_trends.chart_data);
                    initializeStudentPerformanceChart(data.class_trends.chart_data);
                    data.student_trends.forEach((student, index) => {
                        initializeStudentChart(student, index);
                    });
                    
                    dashboardDiv.scrollIntoView({behavior: 'smooth'});
                } catch (error) {
                    dashboardDiv.style.display = 'block';
                    dashboardDiv.className = 'result fail';
                    dashboardDiv.innerHTML = `<p>❌ Error loading dashboard: ${error.message}</p>`;
                }
            }

            // Chart Initialization Functions
            function initializeClassPerformanceChart(chartData) {
                const ctx = document.getElementById('classPerformanceChart').getContext('2d');
                charts.classPerformance = new Chart(ctx, {
                    type: 'line',
                    data: {
                        labels: chartData.weeks,
                        datasets: [
                            {
                                label: 'Avg Study Hours',
                                data: chartData.avg_study_hours,
                                borderColor: '#007bff',
                                backgroundColor: 'rgba(0, 123, 255, 0.1)',
                                tension: 0.4,
                                fill: true
                            },
                            {
                                label: 'Avg Attendance %',
                                data: chartData.avg_attendance,
                                borderColor: '#28a745',
                                backgroundColor: 'rgba(40, 167, 69, 0.1)',
                                tension: 0.4,
                                fill: true
                            }
                        ]
                    },
                    options: {
                        responsive: true,
                        plugins: {
                            title: { display: true, text: 'Class Performance Metrics' },
                            tooltip: { mode: 'index', intersect: false }
                        },
                        scales: {
                            y: { beginAtZero: true, title: { display: true, text: 'Metrics' } }
                        }
                    }
                });
            }

            function initializePassRateChart(chartData) {
                const ctx = document.getElementById('passRateChart').getContext('2d');
                charts.passRate = new Chart(ctx, {
                    type: 'bar',
                    data: {
                        labels: chartData.weeks,
                        datasets: [{
                            label: 'Pass Rate %',
                            data: chartData.pass_rates,
                            backgroundColor: chartData.pass_rates.map((rate, index) => 
                                index > 0 && rate > chartData.pass_rates[index-1] ? '#28a745' : 
                                index > 0 && rate < chartData.pass_rates[index-1] ? '#dc3545' : '#ffc107'
                            ),
                            borderColor: '#343a40',
                            borderWidth: 1
                        }]
                    },
                    options: {
                        responsive: true,
                        plugins: {
                            title: { display: true, text: 'Weekly Pass Rate Evolution' }
                        },
                        scales: {
                            y: { 
                                beginAtZero: true, 
                                max: 100,
                                title: { display: true, text: 'Pass Rate %' }
                            }
                        }
                    }
                });
            }

            function initializeStudentPerformanceChart(chartData) {
                const ctx = document.getElementById('studentPerformanceChart').getContext('2d');
                const sortedStudents = [...chartData.student_performance].sort((a, b) => b.overall_score - a.overall_score);
                
                charts.studentPerformance = new Chart(ctx, {
                    type: 'bar',
                    data: {
                        labels: sortedStudents.map(s => s.name),
                        datasets: [{
                            label: 'Overall Performance Score',
                            data: sortedStudents.map(s => s.overall_score),
                            backgroundColor: sortedStudents.map(s => 
                                s.trend === 'Improving' ? '#28a745' : 
                                s.trend === 'Declining' ? '#dc3545' : '#ffc107'
                            ),
                            borderColor: '#343a40',
                            borderWidth: 1
                        }]
                    },
                    options: {
                        responsive: true,
                        indexAxis: 'y',
                        plugins: {
                            title: { display: true, text: 'Student Performance Ranking' },
                            tooltip: {
                                callbacks: {
                                    afterLabel: function(context) {
                                        const student = sortedStudents[context.dataIndex];
                                        return `Trend: ${student.trend}`;
                                    }
                                }
                            }
                        }
                    }
                });
            }

            function initializeStudentChart(student, index) {
                const ctx = document.getElementById(`studentChart-${index}`).getContext('2d');
                charts[`studentChart-${index}`] = new Chart(ctx, {
                    type: 'line',
                    data: {
                        labels: student.chart_data.labels,
                        datasets: [
                            {
                                label: 'Study Hours',
                                data: student.chart_data.study_hours,
                                borderColor: '#007bff',
                                backgroundColor: 'rgba(0, 123, 255, 0.1)',
                                yAxisID: 'y',
                                tension: 0.4
                            },
                            {
                                label: 'Attendance %',
                                data: student.chart_data.attendance,
                                borderColor: '#28a745',
                                backgroundColor: 'rgba(40, 167, 69, 0.1)',
                                yAxisID: 'y',
                                tension: 0.4
                            },
                            {
                                label: 'Confidence %',
                                data: student.chart_data.confidence,
                                borderColor: '#ffc107',
                                backgroundColor: 'rgba(255, 193, 7, 0.1)',
                                yAxisID: 'y1',
                                tension: 0.4
                            }
                        ]
                    },
                    options: {
                        responsive: true,
                        interaction: { mode: 'index', intersect: false },
                        scales: {
                            y: {
                                type: 'linear',
                                display: true,
                                position: 'left',
                                title: { display: true, text: 'Hours/Attendance' }
                            },
                            y1: {
                                type: 'linear',
                                display: true,
                                position: 'right',
                                title: { display: true, text: 'Confidence %' },
                                grid: { drawOnChartArea: false }
                            }
                        }
                    }
                });
            }

            // KEEP ALL YOUR EXISTING FUNCTIONS (predict, processBatch, showAnalytics, showTips, showModelInfo, loadClassTrends)
            // They remain exactly the same as in your current main.rs

            async function predict() {
                const hours = document.getElementById('hours').value;
                const attendance = document.getElementById('attendance').value;
                const resultDiv = document.getElementById('result');
                
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
                        <h3>Prediction: ${data.prediction}</h3>
                        <p><strong>Confidence:</strong> ${(data.confidence * 100).toFixed(1)}%</p>
                        <p>Student with ${hours} study hours and ${attendance}% attendance is predicted to: <strong>${data.prediction}</strong></p>
                    `;
                } catch (error) {
                    resultDiv.style.display = 'block';
                    resultDiv.className = 'result fail';
                    resultDiv.innerHTML = `<p>Error: ${error.message}</p>`;
                }
            }

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
                    resultDiv.innerHTML = `<p>Error processing batch: ${error.message}</p>`;
                }
            }

            async function showAnalytics() {
                const analyticsDiv = document.getElementById('analytics');
                const contentDiv = document.getElementById('analytics-content');
                
                try {
                    const response = await fetch('/analytics');
                    const data = await response.json();
                    
                    contentDiv.innerHTML = `
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin: 20px 0;">
                            <div style="background: white; padding: 15px; border-radius: 8px; border-left: 4px solid #007bff;">
                                <h4>Total Students</h4>
                                <p style="font-size: 24px; margin: 0; color: #007bff;">${data.total_students}</p>
                            </div>
                            <div style="background: white; padding: 15px; border-radius: 8px; border-left: 4px solid #28a745;">
                                <h4>Overall Pass Rate</h4>
                                <p style="font-size: 24px; margin: 0; color: #28a745;">${(data.pass_rate * 100).toFixed(1)}%</p>
                            </div>
                            <div style="background: white; padding: 15px; border-radius: 8px; border-left: 4px solid #ffc107;">
                                <h4>Avg Study Hours</h4>
                                <p style="font-size: 24px; margin: 0; color: #ffc107;">${data.avg_study_hours}h</p>
                            </div>
                            <div style="background: white; padding: 15px; border-radius: 8px; border-left: 4px solid #dc3545;">
                                <h4>Avg Attendance</h4>
                                <p style="font-size: 24px; margin: 0; color: #dc3545;">${data.avg_attendance}%</p>
                            </div>
                        </div>
                        
                        <h4>Performance Breakdown</h4>
                        ${data.performance_breakdown.map(category => `
                            <div style="background: white; padding: 10px; margin: 8px 0; border-radius: 5px; border-left: 4px solid #6c757d;">
                                <strong>${category.range}</strong>: ${category.count} students | Pass Rate: ${(category.pass_rate * 100).toFixed(1)}%
                            </div>
                        `).join('')}
                    `;
                    
                    analyticsDiv.style.display = 'block';
                    analyticsDiv.scrollIntoView({behavior: 'smooth'});
                } catch (error) {
                    contentDiv.innerHTML = `<p style="color: red;">Error loading analytics: ${error.message}</p>`;
                    analyticsDiv.style.display = 'block';
                }
            }

            async function showTips() {
                const tipsDiv = document.getElementById('tips');
                const contentDiv = document.getElementById('tips-content');
                
                try {
                    const response = await fetch('/tips');
                    const data = await response.json();
                    
                    contentDiv.innerHTML = `
                        <h4>Evidence-Based Study Recommendations:</h4>
                        <ul style="background: white; padding: 15px; border-radius: 5px;">
                            ${data.map(tip => `<li style="margin: 8px 0;">${tip}</li>`).join('')}
                        </ul>
                    `;
                    
                    tipsDiv.style.display = 'block';
                    tipsDiv.scrollIntoView({behavior: 'smooth'});
                } catch (error) {
                    contentDiv.innerHTML = `<p style="color: red;">Error loading tips: ${error.message}</p>`;
                    tipsDiv.style.display = 'block';
                }
            }

            async function showModelInfo() {
                const modelDiv = document.getElementById('model-info');
                const contentDiv = document.getElementById('model-info-content');
                
                try {
                    const response = await fetch('/model/info');
                    const data = await response.json();
                    
                    contentDiv.innerHTML = `
                        <div style="background: white; padding: 15px; border-radius: 5px;">
                            <p><strong>Model Accuracy:</strong> ${(data.accuracy * 100).toFixed(1)}%</p>
                            <p><strong>Algorithm:</strong> Logistic Regression</p>
                            <p><strong>Features:</strong> Study Hours, Attendance Percentage</p>
                            <p><strong>Training Data:</strong> TUK Student Academic Records</p>
                        </div>
                    `;
                    
                    modelDiv.style.display = 'block';
                    modelDiv.scrollIntoView({behavior: 'smooth'});
                } catch (error) {
                    contentDiv.innerHTML = `<p style="color: red;">Error loading model info: ${error.message}</p>`;
                    modelDiv.style.display = 'block';
                }
            }

            async function loadClassTrends() {
                const classDiv = document.getElementById('class-trends');
                const dashboardDiv = document.getElementById('trends-dashboard');
                dashboardDiv.style.display = 'none';
                
                try {
                    const response = await fetch('/class-trends');
                    const data = await response.json();
                    
                    classDiv.innerHTML = `
                        <h3>👥 Class Performance Trends</h3>
                        <div style="background: white; padding: 15px; border-radius: 5px; margin: 15px 0;">
                            <h4>Class Summary</h4>
                            <p><strong>Total Students:</strong> ${data.total_students}</p>
                            <p><strong>Average Improvement:</strong> ${data.average_improvement.toFixed(2)}</p>
                        </div>
                        
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0;">
                            <div style="background: #d4edda; padding: 15px; border-radius: 8px;">
                                <h5>🏆 Top Performers</h5>
                                <ul>
                                    ${data.top_performers.map(student => `<li>${student}</li>`).join('')}
                                </ul>
                            </div>
                            <div style="background: #f8d7da; padding: 15px; border-radius: 8px;">
                                <h5>⚠️ Needs Support</h5>
                                <ul>
                                    ${data.at_risk_students.map(student => `<li>${student}</li>`).join('')}
                                </ul>
                            </div>
                        </div>
                        
                        <h4>Weekly Performance Metrics</h4>
                        <table class="prediction-table">
                            <thead>
                                <tr>
                                    <th>Week</th>
                                    <th>Avg Study Hours</th>
                                    <th>Avg Attendance</th>
                                    <th>Pass Rate</th>
                                    <th>Trend</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${data.weekly_summary.map((week, index) => {
                                    const trend = index > 0 ? 
                                        (week.pass_rate > data.weekly_summary[index-1].pass_rate ? '📈' : 
                                         week.pass_rate < data.weekly_summary[index-1].pass_rate ? '📉' : '➡️') : '➡️';
                                    return `
                                        <tr>
                                            <td>${week.week}</td>
                                            <td>${week.avg_study_hours.toFixed(1)}h</td>
                                            <td>${week.avg_attendance.toFixed(1)}%</td>
                                            <td>${(week.pass_rate * 100).toFixed(1)}%</td>
                                            <td>${trend}</td>
                                        </tr>
                                    `;
                                }).join('')}
                            </tbody>
                        </table>
                    `;
                    
                    classDiv.style.display = 'block';
                    classDiv.scrollIntoView({behavior: 'smooth'});
                } catch (error) {
                    classDiv.style.display = 'block';
                    classDiv.className = 'result fail';
                    classDiv.innerHTML = `<p>Error loading class trends: ${error.message}</p>`;
                }
            }
        </script>
    </body>
    </html>
    "#;

    HttpResponse::Ok().content_type("text/html").body(html_content)
}

// UPDATE: Add new routes to start_api function
async fn start_api(
    model: TrainedModel,
    model_info: ModelInfo,
) -> std::io::Result<()> {
    let model_data = web::Data::new(model);
    let info_data = web::Data::new(model_info);
    
    HttpServer::new(move || {
        App::new()
            .app_data(model_data.clone())
            .app_data(info_data.clone())
            .route("/", web::get().to(serve_homepage))
            .route("/predict", web::post().to(predict))
            .route("/batch-predict", web::post().to(batch_predict))
            .route("/model/info", web::get().to(get_model_info))
            .route("/health", web::get().to(health_check))
            .route("/analytics", web::get().to(get_analytics))
            .route("/tips", web::get().to(get_success_tips))
            .route("/student-trends", web::post().to(get_student_trends))  // NEW
            .route("/class-trends", web::get().to(get_class_trends))       // NEW
            .route("/trends-dashboard", web::get().to(get_trends_dashboard)) // NEW
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Loading TUK student data...");
    
    let data = load_data("data/students.csv")?;
    println!("Loaded {} student records", data.nrows());

    let (model, accuracy) = train_model(data)?;
    println!("🎯 Model trained successfully! Accuracy: {:.2}%", accuracy * 100.0);

    let model_info = ModelInfo { accuracy };

    println!("🌐 Starting TUK Student Predictor API on http://127.0.0.1:8080");
    println!("   Visit http://127.0.0.1:8080 in your browser!");
    println!("   NEW: Interactive Charts Dashboard available!");
    
    start_api(model, model_info).await?;

    Ok(())
}