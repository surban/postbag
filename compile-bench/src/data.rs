// Shared test data constructors for compile-time benchmarking.

use crate::types::*;
use std::collections::BTreeMap;

pub fn simple() -> Simple {
    Simple {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: -1,
        f: -2,
        g: -3,
        h: -4,
        i: 1.0,
        j: 2.0,
        k: true,
        l: "hi".into(),
        m: vec![1, 2, 3],
    }
}

pub fn address() -> Address {
    Address { street: "s".into(), city: "c".into(), state: "s".into(), zip: "z".into(), country: "co".into() }
}

pub fn empty_address() -> Address {
    Address { street: "".into(), city: "".into(), state: "".into(), zip: "".into(), country: "".into() }
}

pub fn person() -> Person {
    Person {
        name: "n".into(),
        age: 1,
        email: None,
        address: address(),
        tags: vec![],
        metadata: BTreeMap::new(),
        favorite_color: Color::Red,
        active: true,
    }
}

pub fn empty_person() -> Person {
    Person {
        name: "".into(),
        age: 0,
        email: None,
        address: empty_address(),
        tags: vec![],
        metadata: BTreeMap::new(),
        favorite_color: Color::Red,
        active: false,
    }
}

pub fn company() -> Company {
    Company {
        name: "c".into(),
        employees: vec![],
        headquarters: empty_address(),
        revenue: 0.0,
        departments: BTreeMap::new(),
    }
}

pub fn config() -> Config {
    Config { version: 1, name: "n".into(), entries: vec![], nested: BTreeMap::new() }
}

pub fn config_entry() -> ConfigEntry {
    ConfigEntry { key: "k".into(), value: ConfigValue::Int(1) }
}

pub fn matrix() -> Matrix {
    Matrix { rows: 0, cols: 0, data: vec![] }
}

pub fn time_series() -> TimeSeries {
    TimeSeries { name: "t".into(), points: vec![], metadata: BTreeMap::new() }
}

pub fn document() -> Document {
    Document {
        title: "t".into(),
        author: empty_person(),
        sections: vec![],
        references: vec![],
        metadata: BTreeMap::new(),
    }
}

pub fn event() -> Event {
    Event { id: 0, timestamp: 0, kind: EventKind::Logout { user_id: 0 } }
}

pub fn api_response() -> ApiResponse {
    ApiResponse { status: 200, headers: BTreeMap::new(), body: ResponseBody::Json("{}".into()) }
}

pub fn database_record() -> DatabaseRecord {
    DatabaseRecord {
        id: 0,
        table: "t".into(),
        fields: BTreeMap::new(),
        relations: vec![],
        version: 0,
        deleted: false,
    }
}

pub fn workspace() -> Workspace {
    Workspace {
        name: "w".into(),
        owner: empty_person(),
        documents: vec![],
        configs: vec![],
        events: vec![],
        records: vec![],
        responses: vec![],
        time_series: vec![],
        matrices: vec![],
        messages: vec![],
    }
}
