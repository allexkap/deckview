use log::{debug, info};
use rusqlite::{Connection, Result, types};

pub type AppId = u32;
pub enum EventType {
    Running = 0,
    Started,
    Stopped,
    Suspended,
    Resumed,
}

impl types::FromSql for EventType {
    fn column_result(value: types::ValueRef<'_>) -> types::FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(EventType::Running),
            1 => Ok(EventType::Started),
            2 => Ok(EventType::Stopped),
            3 => Ok(EventType::Suspended),
            4 => Ok(EventType::Resumed),
            i => Err(rusqlite::types::FromSqlError::OutOfRange(i)),
        } // ._.
    }
}

pub type Sessions = Vec<[u64; 2]>;

pub struct DeckDBv {
    conn: Connection,
}

impl DeckDBv {
    pub fn open(path: &str) -> Result<DeckDBv> {
        let conn = Connection::open(path)?;
        info!("database {path:?} opened successfully");
        Ok(DeckDBv { conn })
    }

    pub fn load_sessions(&self, app_id: AppId, start: u64, stop: u64) -> Result<Sessions> {
        let mut stmt = self.conn.prepare(
            "select timestamp, event_type from events \
                where object_id = ?1 and ?2 <= timestamp and timestamp < ?3",
        )?;

        let mut prev_ts = start;
        let sessions: Sessions = stmt
            .query_map((app_id, start, stop), |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(Result::ok)
            .filter_map(|(ts, event)| match event {
                EventType::Running => None,
                EventType::Started | EventType::Resumed => {
                    prev_ts = ts;
                    None
                }
                EventType::Stopped | EventType::Suspended => Some([prev_ts, ts]),
            })
            .collect();

        debug!(
            "loaded {} session(s) with AppId={app_id} from {start} to {stop}",
            sessions.len(),
        );

        Ok(sessions)
    }
}
