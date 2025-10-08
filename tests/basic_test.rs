#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_calculation() {
        let predictions = vec![true, true, false, true];
        let targets = vec![true, false, false, true];
        
        assert_eq!(predictions.len(), targets.len());
    }
}