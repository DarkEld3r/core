use database::album;
use database::artist;
use diesel::prelude::*;

use context::GraphQLContext;
use juniper::FieldResult;
use models::*;

#[derive(Queryable, Identifiable, Insertable)]
#[table_name = "artist"]
pub struct Artist {
    pub id: UUID,
    pub name: String,
    pub time_added: i32,

    pub last_played: Option<i32>,
}

impl Artist {
    pub fn from_id(context: &GraphQLContext, id: &UUID) -> FieldResult<Self> {
        let conn = context.connection();
        Ok(artist::table.filter(artist::id.eq(id)).first::<Self>(conn)?)
    }

    pub fn albums(&self, context: &GraphQLContext) -> FieldResult<Vec<Album>> {
        let conn = context.connection();
        Ok(album::table
            .filter(album::artist_id.eq(&self.id))
            .order(album::time_added.desc())
            .load::<Album>(conn)?)
    }

    pub fn stats(&self) -> UserStats {
        UserStats {
            id: format!("stats:{}", self.id.to_string()),
            last_played: self.last_played,
        }
    }
}

graphql_object!(Artist: GraphQLContext |&self| {
    field id() -> &UUID {
        &self.id
    }

    field name() -> &str {
        &self.name
    }

    field albums(&executor) -> FieldResult<Vec<Album>> {
        self.albums(executor.context())
    }

    field stats() -> UserStats {
        self.stats()
    }

    field time_added() -> i32 {
        self.time_added
    }
});
