use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub is_student: bool,
    pub hobbies: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NasaImages {
    pub collection: Collection
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Collection {
    pub href: String,
    pub items: Vec<Item>,
    pub links: Option<Vec<Link>>,
    pub metadata: TotalHit,
    pub version: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TotalHit {
    pub total_hits: i32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub data: Vec<Data>,
    pub href: String,
    pub links: Option<Vec<Link>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub center: Option<String>,
    pub date_created: String,
    pub description: String,
    pub keywords: Option<Vec<String>>,
    pub media_type: String,
    pub nasa_id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link {
    pub href: String,
    pub rel: String,
    pub prompt: Option<String>,
    pub render: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserStitches {
    pub id: i32,
    pub search_terms: String,
    pub created_on: String,
}