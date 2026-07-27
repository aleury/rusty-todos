#[derive(Debug, toasty::Model)]
pub struct Todo {
    #[key]
    #[auto]
    pub id: u64,

    pub title: String,

    pub done: bool,

    #[default(jiff::Timestamp::now())]
    pub created_at: jiff::Timestamp,
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}
