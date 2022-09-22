use serde::Deserialize;

#[derive(Deserialize)]
pub struct Event {
    #[serde(alias = "type")]
    event_type: String,
    aggregate_name: String,
    aggregate_id: String,
    #[serde(skip_deserializing)]
    pub payload: String,
}

impl Event {
    pub fn get_insert_query(self) -> String {
        format!(
            "INSERT INTO event(event_type, aggregate_name, aggregate_id, occurred_on, payload, inserted_at, updated_at) VALUES('{}', '{}', '{}', '{}', '{}', '{}', '{}');",
            self.event_type,
            self.aggregate_name,
            self.aggregate_id,
            "2020-01-01",
            self.payload,
            "2020-01-01",
            "2020-01-01"
        )
    }
}
