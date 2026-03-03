// Compile-time benchmark: serde derive only (no codec).
mod types;

use types::*;

fn main() {
    // Ensure all types are used so serde derive is fully instantiated.
    let _: Option<Simple> = None;
    let _: Option<Color> = None;
    let _: Option<Address> = None;
    let _: Option<Person> = None;
    let _: Option<Company> = None;
    let _: Option<Message> = None;
    let _: Option<Config> = None;
    let _: Option<ConfigEntry> = None;
    let _: Option<ConfigValue> = None;
    let _: Option<Matrix> = None;
    let _: Option<TimeSeries> = None;
    let _: Option<Document> = None;
    let _: Option<Section> = None;
    let _: Option<Figure> = None;
    let _: Option<Event> = None;
    let _: Option<EventKind> = None;
    let _: Option<ApiResponse> = None;
    let _: Option<ResponseBody> = None;
    let _: Option<DatabaseRecord> = None;
    let _: Option<Workspace> = None;
    println!("serde derive only OK");
}
