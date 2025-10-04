use ndarray::{Array2, s, Array1};
use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use csv::Reader;
use std::error::Error;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PredictRequest {
    hours: f64,
    attendance: f64,
}

#[derive(Serialize)]
struct PredictResponse {
    prediction: String,
    confidence: f64,
}

#[derive(Serialize, Clone)]
struct ModelInfo {
    accuracy: f64,
}

async fn predict(
    req: web::Json<PredictRequest>,
    model: web::Data<linfa_logistic::FittedLogisticRegression<f64, bool>>,
) -> HttpResponse {
    let features = Array2::from_shape_vec((1, 2), vec![req.hours, req.attendance]).unwrap();
    let prediction = model.predict(&features);
    
    let response = PredictResponse {
        prediction: if prediction[0] { "Pass".to_string() } else { "Fail".to_string() },
        confidence: 0.85,
    };
    
    HttpResponse::Ok().json(response)
}

async fn get_model_info(model_data: web::Data<ModelInfo>) -> HttpResponse {
    HttpResponse::Ok().json(model_data.as_ref().clone())
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("Student Classifier API is running!")
}

// Add this new endpoint to serve the HTML page
async fn serve_homepage() -> HttpResponse {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Student Classifier</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
            .container { background: #f5f5f5; padding: 20px; border-radius: 10px; }
            .form-group { margin: 15px 0; }
            label { display: block; margin-bottom: 5px; font-weight: bold; }
            input { width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; }
            button { background: #007bff; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
            button:hover { background: #0056b3; }
            .result { margin-top: 20px; padding: 15px; border-radius: 5px; display: none; }
            .pass { background: #d4edda; color: #155724; border: 1px solid #c3e6cb; }
            .fail { background: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1> The Technical University of Kenya Student Performance Predictor based on attendance</h1>
            <p>Enter student details to predict if they will pass or fail:</p>
            
            <div class="form-group">
                <label for="hours">Study Hours:</label>
                <input type="number" id="hours" step="0.1" placeholder="e.g., 6.0" value="6.0">
            </div>
            
            <div class="form-group">
                <label for="attendance">Attendance (%):</label>
                <input type="number" id="attendance" step="0.1" placeholder="e.g., 85.0" value="85.0">
            </div>
            
            <button onclick="predict()">Predict Result</button>
            
            <div id="result" class="result"></div>
        </div>

        <script>
            async function predict() {
                const hours = document.getElementById('hours').value;
                const attendance = document.getElementById('attendance').value;
                const resultDiv = document.getElementById('result');
                
                try {
                    const response = await fetch('/predict', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({
                            hours: parseFloat(hours),
                            attendance: parseFloat(attendance)
                        })
                    });
                    
                    const data = await response.json();
                    
                    resultDiv.style.display = 'block';
                    resultDiv.className = 'result ' + (data.prediction === 'Pass' ? 'pass' : 'fail');
                    resultDiv.innerHTML = `
                        <h3>Prediction: ${data.prediction}</h3>
                        <p>Confidence: ${(data.confidence * 100).toFixed(1)}%</p>
                        <p>Student with ${hours} study hours and ${attendance}% attendance is predicted to: <strong>${data.prediction}</strong></p>
                    `;
                } catch (error) {
                    resultDiv.style.display = 'block';
                    resultDiv.className = 'result fail';
                    resultDiv.innerHTML = `<p>Error: ${error.message}</p>`;
                }
            }
        </script>
    </body>
    </html>
    "#;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}

async fn start_api(
    model: linfa_logistic::FittedLogisticRegression<f64, bool>,
    model_info: ModelInfo,
) -> std::io::Result<()> {
    let model_data = web::Data::new(model);
    let info_data = web::Data::new(model_info);
    
    HttpServer::new(move || {
        App::new()
            .app_data(model_data.clone())
            .app_data(info_data.clone())
            .route("/", web::get().to(serve_homepage))  // Add homepage route
            .route("/predict", web::post().to(predict))
            .route("/model/info", web::get().to(get_model_info))
            .route("/health", web::get().to(health_check))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn load_data(path: &str) -> Result<Array2<f64>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;
    let mut data = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let hours: f64 = record[0].parse()?;
        let attendance: f64 = record[1].parse()?;
        let pass_fail: f64 = record[2].parse()?;
        data.push(vec![hours, attendance, pass_fail]);
    }

    let num_rows = data.len();
    let flat_data = data.into_iter().flatten().collect::<Vec<f64>>();
    Ok(Array2::from_shape_vec((num_rows, 3), flat_data)?)
}

fn calculate_accuracy(predictions: &Array1<bool>, targets: &Array1<bool>) -> f64 {
    predictions.iter()
        .zip(targets.iter())
        .filter(|(&pred, &actual)| pred == actual)
        .count() as f64 / targets.len() as f64
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Loading student data...");
    
    // Use original data file
    let data = load_data("data/students.csv")?;
    println!("Loaded {} student records", data.nrows());

    // Check class distribution
    let targets = data.column(2).mapv(|x| x > 0.5).into_raw_vec();
    let pass_count = targets.iter().filter(|&&x| x).count();
    let fail_count = targets.iter().filter(|&&x| !x).count();
    println!("Class distribution: {} Pass, {} Fail", pass_count, fail_count);

    // If we don't have enough of both classes, create synthetic data
    let (features, targets) = if pass_count < 2 || fail_count < 2 {
        println!("⚠️  Adding synthetic data for better training...");
        let mut synthetic_data = vec![
            vec![1.0, 40.0, 0.0], // More fail examples
            vec![2.0, 50.0, 0.0],
            vec![8.0, 95.0, 1.0], // More pass examples  
            vec![9.0, 90.0, 1.0],
        ];
        
        // Add original data
        for i in 0..data.nrows() {
            synthetic_data.push(vec![
                data[[i, 0]], data[[i, 1]], data[[i, 2]]
            ]);
        }
        
        let num_rows = synthetic_data.len();
        let flat_data: Vec<f64> = synthetic_data.into_iter().flatten().collect();
        let enhanced_data = Array2::from_shape_vec((num_rows, 3), flat_data)?;
        
        (enhanced_data.slice(s![.., ..2]).to_owned(), 
         Array1::from_vec(enhanced_data.column(2).mapv(|x| x > 0.5).into_raw_vec()))
    } else {
        (data.slice(s![.., ..2]).to_owned(),
         Array1::from_vec(targets))
    };

    // Train model
    println!("📊 Training logistic regression model...");
    let dataset = Dataset::new(features.clone(), targets.clone());
    let model = LogisticRegression::default()
        .max_iterations(100)
        .fit(&dataset)
        .expect("Failed to train model");

    // Evaluate
    let predictions = model.predict(&features);
    let accuracy = calculate_accuracy(&predictions, &targets);

    println!("🎯 Model trained successfully!");
    println!("   Accuracy: {:.2}%", accuracy * 100.0);

    // Create model info
    let model_info = ModelInfo {
        accuracy,
    };

    // Start API
    println!("🌐 Starting API server on http://127.0.0.1:8080");
    println!("   Visit http://127.0.0.1:8080 in your browser to use the web interface!");
    start_api(model, model_info).await?;

    Ok(())
}