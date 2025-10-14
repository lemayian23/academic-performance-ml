use std::error::Error;
use csv::Reader;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

//function to validate CSV data exists
pub fn validate_data(path: &str) -> Result<usize> {
    let mut rdr = Reader::from_path(path)?;
    let mut count = 0;
    
    for result in rdr.records() {
        let _record = result?;
        count += 1;
    }
    
    Ok(count)
}