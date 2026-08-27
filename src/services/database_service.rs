use mongodb::{bson::Document, Client, Collection};

#[derive(Clone)]
pub struct DatabaseService {
    pub users: Collection<Document>,
    pub watch_progress: Collection<Document>,
}

impl DatabaseService {
    pub async fn new() -> Self {
        let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
        let client = Client::with_uri_str(uri).await.expect("Failed to connect to MongoDB");
        let db = client.database("user_auth");
        DatabaseService {
            users: db.collection("users"),
            watch_progress: db.collection("watch_progress"),
        }
    }
}
