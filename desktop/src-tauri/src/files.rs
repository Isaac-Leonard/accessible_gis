use csv::ReaderBuilder;

#[tauri::command]
#[specta::specta]
pub fn get_csv(file: String) -> Vec<Vec<String>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .expect("Could not read file");
    reader
        .deserialize::<Vec<String>>()
        .map(|r| match r {
            Ok(record) => record,
            Err(err) => {
                panic!("Invalid record: {:?}", err);
            }
        })
        .collect()
}
