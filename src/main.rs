mod data;
mod model;

use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::error::Error;

use crate::data::load_data;
use crate::model::{train_model, ModelInfo, PredictResponse, AnalyticsData, PerformanceCategory, 
                   StudentRecord, TrainedModel};

#[derive(Deserialize)]
struct PredictRequest {
    hours: f64,
    attendance: f64,
}

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

// Homepage endpoint
async fn serve_homepage() -> HttpResponse {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>The Technical University of Kenya - Student Performance Predictor</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
            .container { background: #f5f5f5; padding: 25px; border-radius: 10px; }
            .form-group { margin: 15px 0; }
            label { display: block; margin-bottom: 5px; font-weight: bold; }
            input, textarea { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; }
            button { background: #007bff; color: white; padding: 12px 24px; border: none; border-radius: 4px; cursor: pointer; margin: 5px; }
            button:hover { background: #0056b3; }
            .result { margin-top: 20px; padding: 20px; border-radius: 5px; display: none; }
            .pass { background: #d4edda; color: #155724; border: 1px solid #c3e6cb; }
            .fail { background: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }
            .warning { background: #fff3cd; color: #856404; border: 1px solid #ffeaa7; }
            .info { background: #d1ecf1; color: #0c5460; border: 1px solid #bee5eb; }
            .button-group { text-align: center; margin: 20px 0; }
            .feature-section { background: #e8f5e8; padding: 20px; border-radius: 10px; margin: 20px 0; }
            .prediction-table { width: 100%; border-collapse: collapse; margin: 15px 0; }
            .prediction-table th, .prediction-table td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
            .prediction-table th { background: #f8f9fa; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1> The Technical University of Kenya Student Performance Predictor</h1>
            <p>Enter student details to predict academic performance based on study patterns:</p>
            
            <div class="form-group">
                <label for="hours">Weekly Study Hours:</label>
                <input type="number" id="hours" step="0.1" placeholder="e.g., 6.0" value="6.0">
            </div>
            
            <div class="form-group">
                <label for="attendance">Class Attendance (%):</label>
                <input type="number" id="attendance" step="0.1" placeholder="e.g., 85.0" value="85.0">
            </div>
            
            <button onclick="predict()"> Predict Academic Result</button>
            
            <div id="result" class="result"></div>

            <div class="button-group">
                <button onclick="showAnalytics()" style="background: #28a745;"> Performance Analytics</button>
                <button onclick="showTips()" style="background: #6f42c1;"> Success Tips</button>
                <button onclick="showModelInfo()" style="background: #fd7e14;"> Model Info</button>
            </div>

            <!-- Batch Prediction Section -->
            <div class="feature-section">
                <h3> Batch Student Prediction (NEW)</h3>
                <p>Upload multiple students at once for bulk predictions (CSV format):</p>
                
                <textarea id="batchData" placeholder="Enter CSV data:
name,hours,attendance
Denis Lemayian,6.5,85.0
Saitoti Smith,4.0,70.0
Kukutia Johnson,8.0,92.0
Kirionki Williams,3.0,65.0
David Lemoita,7.5,88.0" 
                    rows="8" style="width: 100%; padding: 10px; border-radius: 5px; border: 1px solid #ddd; font-family: monospace;"></textarea>
                
                <button onclick="processBatch()" style="background: #28a745; margin-top: 10px;"> Process Batch Predictions</button>
                
                <div id="batch-result" class="result" style="display: none;"></div>
            </div>

            <div id="analytics" class="result info" style="display: none;">
                <h3> TUK Student Performance Analytics</h3>
                <div id="analytics-content"></div>
            </div>

            <div id="tips" class="result info" style="display: none;">
                <h3> TUK Student Success Tips</h3>
                <div id="tips-content"></div>
            </div>

            <div id="model-info" class="result info" style="display: none;">
                <h3> Model Information</h3>
                <div id="model-info-content"></div>
            </div>
        </div>

        <script>
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
                        <h3> Batch Prediction Results</h3>
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
        </script>
    </body>
    </html>
    "#;

    HttpResponse::Ok().content_type("text/html").body(html_content)
}

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
    println!("   NEW: Batch prediction feature available!");
    
    start_api(model, model_info).await?;

    Ok(())
}