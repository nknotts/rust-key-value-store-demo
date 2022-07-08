use clap::{Parser, Subcommand};
use rusqlite::Connection;
use std::{collections::HashMap, fmt, fs::File};

type Database = HashMap<String, String>;

/// A fictional versioning CLI
#[derive(Parser)]
#[clap(about = "A fictional versioning CLI", long_about = None, version, about)]
struct Cli {
    database: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Clones repos
    #[clap(arg_required_else_help = true)]
    Add {
        /// The remote to clone
        key: String,

        value: String,
    },
    /// pushes things
    #[clap(arg_required_else_help = true)]
    Remove {
        /// The remote to target
        key: String,
    },
    /// adds things
    List {},
    Init {},
}

fn main() {
    let cli_args = Cli::parse();

    let serializer = create_serializer(&cli_args.database);

    match cli_args.command {
        Commands::Add { key, value } => {
            add_key(&cli_args.database, key, value, serializer.as_ref())
        }
        Commands::Remove { key } => remove_db_key(&cli_args.database, &key, serializer.as_ref()),
        Commands::List {} => list_db(&cli_args.database, serializer.as_ref()),
        Commands::Init {} => init_db(&cli_args.database, serializer.as_ref()),
    }
    .unwrap()
}

#[derive(Debug)]
struct KeyAlreadyExists;

#[derive(Debug)]
pub struct KeyDoesNotExist {
    key: String,
}

#[derive(Debug)]
enum Error {
    IO(std::io::Error),
    SerdeYaml(serde_yaml::Error),
    SerdeJson(serde_json::Error),
    Sql(rusqlite::Error),
    Csv(csv::Error),
    KeyAlreadyExists(KeyAlreadyExists),
    KeyDoesNotExist(KeyDoesNotExist),
}

impl fmt::Display for KeyDoesNotExist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Key '{}' does not exist in db", self.key)
    }
}

type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::SerdeYaml(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::SerdeJson(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::Sql(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}

impl From<KeyDoesNotExist> for Error {
    fn from(err: KeyDoesNotExist) -> Error {
        Error::KeyDoesNotExist(err)
    }
}

impl From<KeyAlreadyExists> for Error {
    fn from(err: KeyAlreadyExists) -> Error {
        Error::KeyAlreadyExists(err)
    }
}

fn create_serializer(fname: &str) -> Box<dyn Serializer> {
    if fname.ends_with(".yml") {
        Box::new(YamlSerializer {})
    } else if fname.ends_with(".json") {
        Box::new(JsonSerializer {})
    } else if fname.ends_with(".db") || fname.ends_with(".sqlite") {
        Box::new(SqliteSerializer {})
    } else if fname.ends_with(".csv") {
        Box::new(CsvSerializer {})
    } else {
        println!("Could not determine serializer, falling back to yaml");
        Box::new(YamlSerializer {})
    }
}

fn list_db(fname: &str, serializer: &dyn Serializer) -> Result<()> {
    let db = serializer.read_from_file(fname)?;
    println!("Database contains {} entries", db.len());
    for entry in db {
        println!(" Key: {:6}, Value: {}", entry.0, entry.1)
    }
    Ok(())
}

fn remove_db_key(fname: &str, key: &str, serializer: &dyn Serializer) -> Result<()> {
    let mut db = serializer.read_from_file(fname)?;
    let res = db.remove(key);
    if res.is_none() {
        return Err(Error::KeyDoesNotExist(KeyDoesNotExist {
            key: key.to_string(),
        }));
    }
    serializer.write_to_file(fname, db)?;
    println!("Successfully removed key '{}'", key);
    Ok(())
}

fn init_db(fname: &str, serializer: &dyn Serializer) -> Result<()> {
    let mut db = Database::new();
    db.insert("hat".to_string(), "fedora".to_string());
    db.insert("food".to_string(), "hotdog".to_string());
    serializer.write_to_file(fname, db)
}

fn add_key(fname: &str, key: String, value: String, serializer: &dyn Serializer) -> Result<()> {
    let mut db = serializer.read_from_file(fname)?;
    let res = db.insert(key.clone(), value.clone());
    if res.is_some() {
        return Err(Error::KeyAlreadyExists(KeyAlreadyExists {}));
    }
    serializer.write_to_file(fname, db)?;
    println!("Successfully added key/value {}:{}", key, value);
    Ok(())
}

trait Serializer {
    fn write_to_file(&self, fname: &str, db: Database) -> Result<()>;
    fn read_from_file(&self, fname: &str) -> Result<Database>;
}

pub struct YamlSerializer {}

impl Serializer for YamlSerializer {
    fn write_to_file(&self, fname: &str, db: Database) -> Result<()> {
        let s = serde_yaml::to_string(&db)?;
        std::fs::write(fname, s)?;
        Ok(())
    }

    fn read_from_file(&self, fname: &str) -> Result<Database> {
        let yaml_str = std::fs::read_to_string(fname)?;
        let db: Database = serde_yaml::from_str(&yaml_str)?;
        Ok(db)
    }
}

pub struct JsonSerializer {}

impl Serializer for JsonSerializer {
    fn write_to_file(&self, fname: &str, db: Database) -> Result<()> {
        let s = serde_json::to_string(&db)?;
        std::fs::write(fname, s)?;
        Ok(())
    }

    fn read_from_file(&self, fname: &str) -> Result<Database> {
        let yaml_str = std::fs::read_to_string(fname)?;
        let db: Database = serde_json::from_str(&yaml_str)?;
        Ok(db)
    }
}

pub struct SqliteSerializer {}

impl Serializer for SqliteSerializer {
    fn write_to_file(&self, fname: &str, db: Database) -> Result<()> {
        let mut conn = Connection::open(fname)?;
        let t = conn.transaction()?;
        {
            t.execute("DROP TABLE IF EXISTS kvstore", [])?;

            t.execute(
                "CREATE TABLE kvstore(
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    key TEXT UNIQUE NOT NULL,
                    value TEXT NOT NULL
                )",
                [],
            )?;

            let mut insert_stmt = t.prepare("INSERT INTO kvstore (key, value) VALUES (?,?)")?;
            for kv in db {
                insert_stmt.execute(&[kv.0.as_str(), kv.1.as_str()])?;
            }
        }
        t.commit()?;

        Ok(())
    }

    fn read_from_file(&self, fname: &str) -> Result<Database> {
        let conn = Connection::open(fname)?;

        let mut stmt = conn.prepare("SELECT key, value FROM kvstore")?;
        let kv_iter = stmt.query_map([], |row| {
            Ok((row.get::<usize, String>(0)?, row.get::<usize, String>(1)?))
        })?;

        let mut db = Database::new();
        for row in kv_iter {
            let kv = row?;
            let res = db.insert(kv.0, kv.1);
            if res.is_some() {
                return Err(Error::KeyAlreadyExists(KeyAlreadyExists {}));
            }
        }
        Ok(db)
    }
}

pub struct CsvSerializer {}

impl Serializer for CsvSerializer {
    fn write_to_file(&self, fname: &str, db: Database) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(File::create(fname)?);
        wtr.write_record(&["key", "value"])?;
        for row in db {
            wtr.write_record(&[row.0, row.1])?;
        }
        Ok(())
    }

    fn read_from_file(&self, fname: &str) -> Result<Database> {
        let mut rdr = csv::Reader::from_reader(File::open(fname)?);
        let mut db = Database::new();
        for row in rdr.records() {
            let kv = row?;
            assert_eq!(kv.len(), 2);
            let res = db.insert(kv[0].to_string(), kv[1].to_string());
            if res.is_some() {
                return Err(Error::KeyAlreadyExists(KeyAlreadyExists {}));
            }
        }
        Ok(db)
    }
}
