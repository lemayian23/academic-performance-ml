#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_calculation() {
        let predictions = vec![true, true, false, true];
        let targets = vec![true, false, false, true];
        
        // This would test your accuracy function
        assert_eq!(predictions.len(), targets.len());
    }

    #[test] 
    fn test_model_info_creation() {
        let info = ModelInfo { accuracy: 0.85 };
        assert!(info.accuracy >= 0.0 && info.accuracy <= 1.0);
    }
}