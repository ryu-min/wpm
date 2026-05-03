use rusqlite::{Connection, Result};

pub struct WordsetDb {
    conn: Connection,
}

impl std::fmt::Debug for WordsetDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WordsetDb").finish()
    }
}

impl WordsetDb {
    pub fn new() -> Result<Self> {
        let data_dir = directories::ProjectDirs::from("com", "wpm", "app")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        
        std::fs::create_dir_all(&data_dir).ok();
        
        let db_path = data_dir.join("wordset.db");
        let conn = Connection::open(&db_path)?;
        
        let db = Self { conn };
        db.init_tables()?;
        db.seed_data()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS word_sets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                language TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                words TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    fn seed_data(&self) -> Result<()> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM word_sets",
            [],
            |row| row.get(0),
        )?;

        if count > 0 {
            return Ok(());
        }

        let word_files = vec![
            ("en_1000", "en", include_str!("../resources/en_1000.txt")),
            ("en_10000", "en", include_str!("../resources/en_10000.txt")),
            ("ru_5000", "ru", include_str!("../resources/ru_5000.txt")),
            ("ru_10000", "ru", include_str!("../resources/ru_10000.txt")),
            ("ru_50000", "ru", include_str!("../resources/ru_50000.txt")),
        ];

        let mut stmt = self.conn.prepare("INSERT INTO word_sets (name, language, word_count, words) VALUES (?1, ?2, ?3, ?4)")?;

        for (name, lang, content) in word_files {
            let words: Vec<&str> = content.lines().filter(|s| !s.is_empty()).collect();
            let word_count = words.len() as i64;
            let words_blob = words.join(" ");

            stmt.execute((name, lang, word_count, words_blob))?;
        }

        Ok(())
    }

    pub fn get_wordset_names(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM word_sets ORDER BY language, word_count")?;
        let names = stmt.query_map([], |row| row.get(0))?;
        names.collect()
    }

    pub fn get_words(&self, wordset_name: &str) -> Result<Vec<String>> {
        let words_blob: String = self.conn.query_row(
            "SELECT words FROM word_sets WHERE name = ?1",
            [wordset_name],
            |row| row.get(0),
        )?;
        Ok(words_blob.split_whitespace().map(|s| s.to_string()).collect())
    }

    pub fn get_shuffled_words(&self, wordset_name: &str) -> Result<Vec<String>> {
        let mut words = self.get_words(wordset_name)?;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let seed = hasher.finish() as usize;
        let mut rng = seed_rng(seed);
        shuffle(&mut words, &mut rng);
        Ok(words)
    }

    pub fn quick_start_words(&self) -> Result<Vec<String>> {
        let mut words = self.get_words("en_1000")?;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let seed = hasher.finish() as usize;
        let mut rng = seed_rng(seed);
        shuffle(&mut words, &mut rng);
        Ok(words)
    }
}

fn seed_rng(seed: usize) -> Rand {
    Rand { state: seed }
}

struct Rand { state: usize }

impl Rand {
    fn next(&mut self) -> usize {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state >> 16
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut Rand) {
    for i in (1..v.len()).rev() {
        let j = rng.next() % (i + 1);
        v.swap(i, j);
    }
}