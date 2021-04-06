use sqlx::{Connection, Sqlite, SqliteConnection};

const BOT_DB_NAME: &str = "sqlite:bot.db";

pub type Database = SqliteConnection;
pub async fn connect(migrate: bool) -> sqlx::Result<Database> {
    let mut db = sqlx::SqliteConnection::connect(BOT_DB_NAME).await?;

    if migrate {
        sqlx::migrate!().run(&mut db).await?;
    }

    Ok(db)
}

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Command {
    id: i64,
    name: String,
    pub code: String,
}

impl Command {
    ///
    /// * name - channel name (usually broadcaster login name)
    /// * prefix - command prefix (defaults to "!")
    pub fn new(name: String, code: String) -> Command { Command { id: -1, name, code } }

    #[inline]
    pub fn id(&self) -> i64 { self.id }

    #[inline]
    pub fn name(&self) -> &String { &self.name }

    #[inline]
    pub fn is_saved(&self) -> bool { self.id != -1 }

    // TODO: this is bad, just have two methods instead (insert/update)
    // and query the DB to see if the command exists.
    /// Save a channel
    ///
    /// This will insert a new channel or update the existing one
    pub async fn save(&mut self, db: &mut Database) -> sqlx::Result<()> {
        if self.is_saved() {
            sqlx::query::<Sqlite>("UPDATE command SET code=$1 WHERE id=$2")
                .bind(&self.code)
                .bind(self.id)
                .execute(&mut *db)
                .await?;
        } else {
            self.id = sqlx::query::<Sqlite>("INSERT INTO command (name, code) VALUES ($1, $2)")
                .bind(&self.name)
                .bind(&self.code)
                .execute(&mut *db)
                .await?
                .last_insert_rowid();
        }
        Ok(())
    }

    pub async fn delete(&mut self, db: &mut Database) -> sqlx::Result<()> {
        if self.is_saved() {
            sqlx::query::<Sqlite>("DELETE FROM command WHERE id=$1")
                .bind(self.id)
                .execute(&mut *db)
                .await?;
            self.id = -1;
        }

        Ok(())
    }

    /// Fetch all commands from the database
    pub async fn all(db: &mut Database) -> sqlx::Result<Vec<Command>> {
        sqlx::query_as::<Sqlite, Command>("SELECT * FROM command")
            .fetch_all(db)
            .await
    }
}

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Channel {
    id: i64,
    pub name: String,
    pub prefix: String,
    pub joined: i64,
}

impl Channel {
    ///
    /// * name - channel name (usually broadcaster login name)
    /// * prefix - command prefix (defaults to "!")
    pub fn new(name: String, prefix: Option<String>) -> Channel {
        Channel {
            id: -1,
            name,
            prefix: prefix.unwrap_or_else(|| "!".into()),
            joined: 1,
        }
    }

    #[inline]
    pub fn id(&self) -> i64 { self.id }

    #[inline]
    pub fn is_saved(&self) -> bool { self.id != -1 }

    /// Save a channel
    ///
    /// This will insert a new channel or update the existing one
    pub async fn save(&mut self, db: &mut Database) -> sqlx::Result<()> {
        if self.is_saved() {
            sqlx::query::<Sqlite>("UPDATE channel SET name=$1, prefix=$2, joined=$3 WHERE id=$4")
                .bind(&self.name)
                .bind(&self.prefix)
                .bind(self.joined)
                .bind(self.id)
                .execute(&mut *db)
                .await?;
        } else {
            self.id = sqlx::query::<Sqlite>("INSERT INTO channel (name, prefix, joined) VALUES ($1, $2, $3)")
                .bind(&self.name)
                .bind(&self.prefix)
                .bind(self.joined)
                .execute(&mut *db)
                .await?
                .last_insert_rowid();
        }
        Ok(())
    }

    /// Fetch all channels from the database
    pub async fn all(db: &mut Database) -> sqlx::Result<Vec<Channel>> {
        sqlx::query_as::<Sqlite, Channel>("SELECT * FROM channel")
            .fetch_all(db)
            .await
    }
}
