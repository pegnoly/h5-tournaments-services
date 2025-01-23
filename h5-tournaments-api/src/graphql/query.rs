pub struct Query;

#[async_graphql::Object]
impl Query {
    async fn users(&self) -> Result<i32, String> {
        Ok(10)
    } 
}