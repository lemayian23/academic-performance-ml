use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use ndarray::Array2;

pub fn train_model(features: Array2<f64>, targets: Vec<bool>) -> FittedLogisticRegression<f64, bool> {
    let dataset = Dataset::new(features, targets.into());
    LogisticRegression::default()
        .max_iterations(100)
        .fit(&dataset)
        .expect("Failed to train model")
}

pub fn predict(model: &FittedLogisticRegression<f64, bool>, features: Array2<f64>) -> Vec<bool> {
    model.predict(&features)
}