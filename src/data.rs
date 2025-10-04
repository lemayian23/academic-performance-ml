use ndarray::Array2;
use csv::Reader;
use std::error::Error;

pub fn load_data(path: &str) -> Result<Array2<f64>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;
    let mut data = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let hours: f64 = record[0].parse()?;
        let attendance: f64 = record[1].parse()?;
        let pass: f64 = record[2].parse()?;
        data.push(vec![hours, attendance, pass]);
    }
    let num_rows = data.len();
    let flat_data = data.into_iter().flatten().collect::<Vec<f64>>();
    Ok(Array2::from_shape_vec((num_rows, 3), flat_data)?)
}