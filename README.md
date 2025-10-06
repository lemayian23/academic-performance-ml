# The Technical University of Kenya Student Performance Predictor

A machine learning web application built in Rust that predicts student academic performance based on attendance patterns.

## 🎯 Project Overview

This project uses logistic regression to predict whether a student will pass or fail based on:
- **Study hours** per week
- **Attendance percentage** in classes

## 🚀 Features

- **Machine Learning Model**: Logistic regression using Linfa crate
- **REST API**: Actix-web backend with JSON endpoints  
- **Web Interface**: Beautiful HTML frontend for easy predictions
- **Real-time Predictions**: Instant pass/fail predictions with confidence scores

## 📊 Model Performance

- Trained on student academic data
- Real-time prediction API
- Accuracy metrics and model information endpoints

## 📈 Recent Updates

- **Student Success Tips API** - Get evidence-based study recommendations
- **Performance Analytics Dashboard** - View TUK student performance insights
- **Enhanced Web Interface** - Better user experience for predictions
## 🛠️ Tech Stack

- **Backend**: Rust, Actix-web
- **ML**: Linfa, Linfa-logistic, NDArray
- **Data**: CSV processing
- **Frontend**: HTML, CSS, JavaScript

## 🏃‍♂️ Quick Start

```bash
# Clone repository
git clone <your-repo-url>
cd student-performance-predictor

# Run the application
cargo run

📁 Project Structure

src/
├── main.rs          # Main application logic
data/
├── students.csv     # Training dataset
Cargo.toml          # Dependencies
README.md           # This file

📝 License
MIT License - The Technical University of Kenya


## **3. Initial Git Commands & Commit Message**

```bash
# Initialize git repository
git init

# Add all files
git add .

# Initial commit with meaningful message
git commit -m "feat: Initial commit - TUK Student Performance Predictor

- Implement logistic regression ML model for pass/fail prediction
- Add Actix-web REST API with JSON endpoints  
- Create web interface for easy student performance checking
- Include model training and evaluation pipeline
- Support study hours and attendance based predictions
- Ready for deployment and further development"

# Add remote repository (choose one platform)
# For GitHub:
git remote add origin https://github.com/yourusername/tuk-student-predictor.git

# For GitLab:
git remote add origin https://gitlab.com/yourusername/tuk-student-predictor.git

# Push to remote
git push -u origin main