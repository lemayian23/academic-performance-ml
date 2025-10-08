use ndarray::{Array2, Array1, s};
use csv::Reader;
use std::error::Error;

pub fn load_data(path: &str) -> Result<Array2<f64>, Box<dyn Error>> {
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

pub fn calculate_accuracy(predictions: &Array1<bool>, targets: &Array1<bool>) -> f64 {
    predictions.iter()
        .zip(targets.iter())
        .filter(|(&pred, &actual)| pred == actual)
        .count() as f64 / targets.len() as f64
}

pub fn enhance_data_if_needed(data: &Array2<f64>) -> Result<(Array2<f64>, Array1<bool>), Box<dyn Error>> {
    let targets = data.column(2).mapv(|x| x > 0.5).into_raw_vec();
    let pass_count = targets.iter().filter(|&&x| x).count();
    let fail_count = targets.iter().filter(|&&x| !x).count();
    
    println!("Class distribution: {} Pass, {} Fail", pass_count, fail_count);

    if pass_count < 2 || fail_count < 2 {
        println!(" Adding synthetic data for better training...");
        let mut synthetic_data = vec![
            vec![1.0, 40.0, 0.0],
            vec![2.0, 50.0, 0.0],
            vec![8.0, 95.0, 1.0],
            vec![9.0, 90.0, 1.0],
        ];
        
        for i in 0..data.nrows() {
            synthetic_data.push(vec![data[[i, 0]], data[[i, 1]], data[[i, 2]]]);
        }
        
        let num_rows = synthetic_data.len();
        let flat_data: Vec<f64> = synthetic_data.into_iter().flatten().collect();
        let enhanced_data = Array2::from_shape_vec((num_rows, 3), flat_data)?;
        
        Ok((enhanced_data.slice(s![.., ..2]).to_owned(), 
            Array1::from_vec(enhanced_data.column(2).mapv(|x| x > 0.5).into_raw_vec())))
    } else {
        Ok((data.slice(s![.., ..2]).to_owned(), Array1::from_vec(targets)))
    }
}