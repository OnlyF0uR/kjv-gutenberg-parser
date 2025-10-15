use bincode::{Decode, Encode};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

type VerseMap = BTreeMap<String, String>; // verse_number -> text
type ChapterMap = BTreeMap<String, VerseMap>; // chapter_number -> verses

#[derive(Encode, Decode)]
struct Bible {
    ot: BTreeMap<String, ChapterMap>, // Old Testament
    nt: BTreeMap<String, ChapterMap>, // New Testament
}

fn is_book_line(line: &str) -> Option<(String, bool)> {
    let line = line.trim();

    // Skip "Otherwise Called:" lines and similar
    if line.contains("Otherwise Called") || line.is_empty() {
        return None;
    }

    // Check exact matches first (for lines that are complete book titles)
    if line == "The First Book of Samuel" {
        return Some(("1 Samuel".to_string(), true));
    }
    if line == "The Second Book of Samuel" {
        return Some(("2 Samuel".to_string(), true));
    }

    // Check for simple one-word book names that appear alone on a line
    // These are the minor prophets in the OT
    match line {
        "Hosea" => return Some(("Hosea".to_string(), true)),
        "Joel" => return Some(("Joel".to_string(), true)),
        "Amos" => return Some(("Amos".to_string(), true)),
        "Obadiah" => return Some(("Obadiah".to_string(), true)),
        "Jonah" => return Some(("Jonah".to_string(), true)),
        "Micah" => return Some(("Micah".to_string(), true)),
        "Nahum" => return Some(("Nahum".to_string(), true)),
        "Habakkuk" => return Some(("Habakkuk".to_string(), true)),
        "Zephaniah" => return Some(("Zephaniah".to_string(), true)),
        "Haggai" => return Some(("Haggai".to_string(), true)),
        "Zechariah" => return Some(("Zechariah".to_string(), true)),
        "Malachi" => return Some(("Malachi".to_string(), true)),
        _ => {} // Continue to check prefixes
    }

    // Old Testament books - checking in order with flexible matching
    let ot_books = [
        ("The First Book of Moses:", "Genesis"),
        ("The Second Book of Moses:", "Exodus"),
        ("The Third Book of Moses:", "Leviticus"),
        ("The Fourth Book of Moses:", "Numbers"),
        ("The Fifth Book of Moses:", "Deuteronomy"),
        ("The Book of Joshua", "Joshua"),
        ("The Book of Judges", "Judges"),
        ("The Book of Ruth", "Ruth"),
        ("The First Book of the Chronicles", "1 Chronicles"),
        ("The Second Book of the Chronicles", "2 Chronicles"),
        ("Ezra", "Ezra"),
        ("The Book of Nehemiah", "Nehemiah"),
        ("The Book of Esther", "Esther"),
        ("The Book of Job", "Job"),
        ("The Book of Psalms", "Psalms"),
        ("The Proverbs", "Proverbs"),
        ("Ecclesiastes", "Ecclesiastes"),
        ("The Song of Solomon", "Song of Solomon"),
        ("The Book of the Prophet Isaiah", "Isaiah"),
        ("The Book of the Prophet Jeremiah", "Jeremiah"),
        ("The Lamentations of Jeremiah", "Lamentations"),
        ("The Book of the Prophet Ezekiel", "Ezekiel"),
        ("The Book of Daniel", "Daniel"),
    ];

    for (prefix, canonical) in &ot_books {
        if line.starts_with(prefix) {
            return Some((canonical.to_string(), true));
        }
    }

    // Check Kings books AFTER all others to avoid false matches
    if line.starts_with("The First Book of the Kings") {
        return Some(("1 Kings".to_string(), true));
    }
    if line.starts_with("The Second Book of the Kings") {
        return Some(("2 Kings".to_string(), true));
    }

    // New Testament books
    let nt_books = [
        ("The Gospel According to Saint Matthew", "Matthew"),
        ("The Gospel According to Saint Mark", "Mark"),
        ("The Gospel According to Saint Luke", "Luke"),
        ("The Gospel According to Saint John", "John"),
        ("The Acts of the Apostles", "Acts"),
        ("The Epistle of Paul the Apostle to the Romans", "Romans"),
        (
            "The First Epistle of Paul the Apostle to the Corinthians",
            "1 Corinthians",
        ),
        (
            "The Second Epistle of Paul the Apostle to the Corinthians",
            "2 Corinthians",
        ),
        (
            "The Epistle of Paul the Apostle to the Galatians",
            "Galatians",
        ),
        (
            "The Epistle of Paul the Apostle to the Ephesians",
            "Ephesians",
        ),
        (
            "The Epistle of Paul the Apostle to the Philippians",
            "Philippians",
        ),
        (
            "The Epistle of Paul the Apostle to the Colossians",
            "Colossians",
        ),
        (
            "The First Epistle of Paul the Apostle to the Thessalonians",
            "1 Thessalonians",
        ),
        (
            "The Second Epistle of Paul the Apostle to the Thessalonians",
            "2 Thessalonians",
        ),
        (
            "The First Epistle of Paul the Apostle to Timothy",
            "1 Timothy",
        ),
        (
            "The Second Epistle of Paul the Apostle to Timothy",
            "2 Timothy",
        ),
        ("The Epistle of Paul the Apostle to Titus", "Titus"),
        ("The Epistle of Paul the Apostle to Philemon", "Philemon"),
        ("The Epistle of Paul the Apostle to the Hebrews", "Hebrews"),
        ("The General Epistle of James", "James"),
        ("The First Epistle General of Peter", "1 Peter"),
        ("The Second General Epistle of Peter", "2 Peter"),
        ("The First Epistle General of John", "1 John"),
        ("The Second Epistle General of John", "2 John"),
        ("The Third Epistle General of John", "3 John"),
        ("The General Epistle of Jude", "Jude"),
        ("The Revelation of Saint John the Divine", "Revelation"),
    ];

    for (prefix, canonical) in &nt_books {
        if line.starts_with(prefix) {
            return Some((canonical.to_string(), false));
        }
    }

    None
}

fn parse_gutenberg(txt: &str) -> Bible {
    let mut bible = Bible {
        ot: BTreeMap::new(),
        nt: BTreeMap::new(),
    };

    let mut current_book = String::new();
    let mut is_ot = true;
    let mut current_chapter = String::new();
    let mut current_verse_num = String::new();
    let mut current_verse_text = String::new();

    let mut in_bible = false;
    let mut found_books = std::collections::HashSet::new();
    let mut last_line_was_book = false;

    for line in txt.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Skip header until the start
        if !in_bible {
            if line.contains("*** START OF THE PROJECT GUTENBERG") {
                in_bible = true;
            }
            continue;
        }

        // Check for book line first
        if let Some((book, is_old_testament)) = is_book_line(line) {
            // Handle Kings/Samuel edge case
            if last_line_was_book && (book == "1 Kings" || book == "2 Kings") {
                eprintln!("Skipping {} as it's part of Samuel header", book);
                continue;
            }

            if (book == "1 Kings" || book == "2 Kings")
                && current_book.contains("Samuel")
                && current_verse_num.is_empty()
            {
                eprintln!(
                    "Preventing switch from {} to {} (no verses yet)",
                    current_book, book
                );
                continue;
            }

            if !found_books.contains(&book) {
                eprintln!("Found new book: '{}' from line: '{}'", book, line);
                found_books.insert(book.clone());
            } else {
                eprintln!("Re-encountered book: '{}' from line: '{}'", book, line);
            }

            // Save previous verse if exists
            if !current_verse_num.is_empty()
                && !current_book.is_empty()
                && !current_verse_text.is_empty()
            {
                let testament_map = if is_ot { &mut bible.ot } else { &mut bible.nt };
                testament_map
                    .entry(current_book.clone())
                    .or_insert_with(BTreeMap::new)
                    .entry(current_chapter.clone())
                    .or_insert_with(BTreeMap::new)
                    .insert(
                        current_verse_num.clone(),
                        current_verse_text.trim().to_string(),
                    );
            }

            current_book = book;
            is_ot = is_old_testament;
            current_chapter.clear();
            current_verse_num.clear();
            current_verse_text.clear();
            last_line_was_book = true;
            continue;
        }

        last_line_was_book = false;

        // Look for verse references anywhere in the line
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut found_verse_ref = false;

        for (i, word) in words.iter().enumerate() {
            if let Some((ch, v)) = word.split_once(':') {
                if ch.parse::<u32>().is_ok() && v.parse::<u32>().is_ok() {
                    // Found a verse reference!

                    // First, save the previous verse if we have one
                    if !current_verse_num.is_empty() && !current_book.is_empty() {
                        // Append any text before this verse reference to the current verse
                        if i > 0 {
                            if !current_verse_text.is_empty() {
                                current_verse_text.push(' ');
                            }
                            current_verse_text.push_str(&words[..i].join(" "));
                        }

                        let testament_map = if is_ot { &mut bible.ot } else { &mut bible.nt };
                        testament_map
                            .entry(current_book.clone())
                            .or_insert_with(BTreeMap::new)
                            .entry(current_chapter.clone())
                            .or_insert_with(BTreeMap::new)
                            .insert(
                                current_verse_num.clone(),
                                current_verse_text.trim().to_string(),
                            );
                    }

                    // Start the new verse
                    current_chapter = ch.to_string();
                    current_verse_num = v.to_string();

                    // Collect text after the verse reference on this line
                    if i + 1 < words.len() {
                        current_verse_text = words[i + 1..].join(" ");
                    } else {
                        current_verse_text = String::new();
                    }

                    found_verse_ref = true;
                    break;
                }
            }
        }

        // If no verse reference found, this is continuation text for current verse
        if !found_verse_ref && !current_verse_num.is_empty() {
            if !current_verse_text.is_empty() {
                current_verse_text.push(' ');
            }
            current_verse_text.push_str(line);
        }
    }

    // Save the last verse
    if !current_verse_num.is_empty() && !current_book.is_empty() && !current_verse_text.is_empty() {
        let testament_map = if is_ot { &mut bible.ot } else { &mut bible.nt };
        testament_map
            .entry(current_book.clone())
            .or_insert_with(BTreeMap::new)
            .entry(current_chapter)
            .or_insert_with(BTreeMap::new)
            .insert(current_verse_num, current_verse_text.trim().to_string());
    }

    eprintln!(
        "\nFinal book counts - OT: {}, NT: {}",
        bible.ot.len(),
        bible.nt.len()
    );
    eprintln!("OT books: {:?}", bible.ot.keys().collect::<Vec<_>>());

    bible
}

fn write_bible_to_bin(bible: &Bible, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::create(path)?;
    let mut writer = BufWriter::new(f);

    let config = bincode::config::standard();
    bincode::encode_into_std_write(bible, &mut writer, config)?;

    // CRITICAL: Flush the buffer before dropping
    writer.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the full Gutenberg KJV text file
    let file = File::open("pg10.txt")?;
    let reader = BufReader::new(file);
    let txt = reader.lines().collect::<Result<Vec<_>, _>>()?.join("\n");

    // Parse the Bible
    let bible = parse_gutenberg(&txt);

    // Debug: Print all book names
    println!("Parsed {} OT books:", bible.ot.len());
    for (i, book) in bible.ot.keys().enumerate() {
        println!("  {}. {}", i + 1, book);
    }

    println!("\nParsed {} NT books:", bible.nt.len());
    for (i, book) in bible.nt.keys().enumerate() {
        println!("  {}. {}", i + 1, book);
    }

    write_bible_to_bin(&bible, "bible.bin")?;

    println!("\nParsed KJV and saved to bible.bin successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // if not exists we write the binary
    // yet always return bible
    fn get_bible() -> Bible {
        let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let bin_path = format!("{}/bible.bin", root_dir);
        if std::path::Path::new(&bin_path).exists() {
            // Read existing binary
            let data = std::fs::read(&bin_path).expect("Failed to read bible.bin");
            let config = bincode::config::standard();
            let (bible, _): (Bible, usize) =
                bincode::decode_from_slice(&data, config).expect("Failed to decode bible.bin");
            bible
        } else {
            // Parse and write new binary
            let file = File::open("pg10.txt").expect("Failed to open pg10.txt");
            let reader = BufReader::new(file);
            let txt = reader
                .lines()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .join("\n");
            let bible = parse_gutenberg(&txt);
            write_bible_to_bin(&bible, &bin_path).expect("Failed to write bible.bin");
            bible
        }
    }

    #[test]
    fn test_parse_gutenberg() {
        let file = File::open("pg10.txt").expect("Failed to open pg10.txt");
        let reader = BufReader::new(file);
        let txt = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .join("\n");
        let bible = parse_gutenberg(&txt);
        assert_eq!(bible.ot.len(), 39, "Should have 39 OT books");
        assert_eq!(bible.nt.len(), 27, "Should have 27 NT books");

        let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let bin_path = format!("{}/bible.bin", root_dir);
        write_bible_to_bin(&bible, &bin_path).expect("Failed to write bible.bin");
    }

    #[test]
    fn verify_bible_binary() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // Verify some known verses
        assert_eq!(
            bible.ot["Genesis"]["1"]["1"],
            "In the beginning God created the heaven and the earth."
        );

        // Verify chapter counts
        assert_eq!(
            bible.ot["Genesis"].len(),
            50,
            "Genesis should have 50 chapters"
        );
        assert_eq!(
            bible.nt["Matthew"].len(),
            28,
            "Matthew should have 28 chapters"
        );

        // Matthew 3:1
        assert_eq!(
            bible.nt["Matthew"]["3"]["1"],
            "In those days came John the Baptist, preaching in the wilderness of Judaea,"
        );

        Ok(())
    }

    #[test]
    fn verify_all_chapter_counts() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // Old Testament chapter counts
        let ot_chapters = [
            ("Genesis", 50),
            ("Exodus", 40),
            ("Leviticus", 27),
            ("Numbers", 36),
            ("Deuteronomy", 34),
            ("Joshua", 24),
            ("Judges", 21),
            ("Ruth", 4),
            ("1 Samuel", 31),
            ("2 Samuel", 24),
            ("1 Kings", 22),
            ("2 Kings", 25),
            ("1 Chronicles", 29),
            ("2 Chronicles", 36),
            ("Ezra", 10),
            ("Nehemiah", 13),
            ("Esther", 10),
            ("Job", 42),
            ("Psalms", 150),
            ("Proverbs", 31),
            ("Ecclesiastes", 12),
            ("Song of Solomon", 8),
            ("Isaiah", 66),
            ("Jeremiah", 52),
            ("Lamentations", 5),
            ("Ezekiel", 48),
            ("Daniel", 12),
            ("Hosea", 14),
            ("Joel", 3),
            ("Amos", 9),
            ("Obadiah", 1),
            ("Jonah", 4),
            ("Micah", 7),
            ("Nahum", 3),
            ("Habakkuk", 3),
            ("Zephaniah", 3),
            ("Haggai", 2),
            ("Zechariah", 14),
            ("Malachi", 4),
        ];

        for (book, expected_chapters) in &ot_chapters {
            let actual_chapters = bible.ot[*book].len();
            if actual_chapters != *expected_chapters {
                eprintln!(
                    "\n{} has {} chapters (expected {}). Chapters: {:?}",
                    book,
                    actual_chapters,
                    expected_chapters,
                    bible.ot[*book].keys().collect::<Vec<_>>()
                );
            }
            assert_eq!(
                actual_chapters, *expected_chapters,
                "{} should have {} chapters",
                book, expected_chapters
            );
        }

        // New Testament chapter counts
        let nt_chapters = [
            ("Matthew", 28),
            ("Mark", 16),
            ("Luke", 24),
            ("John", 21),
            ("Acts", 28),
            ("Romans", 16),
            ("1 Corinthians", 16),
            ("2 Corinthians", 13),
            ("Galatians", 6),
            ("Ephesians", 6),
            ("Philippians", 4),
            ("Colossians", 4),
            ("1 Thessalonians", 5),
            ("2 Thessalonians", 3),
            ("1 Timothy", 6),
            ("2 Timothy", 4),
            ("Titus", 3),
            ("Philemon", 1),
            ("Hebrews", 13),
            ("James", 5),
            ("1 Peter", 5),
            ("2 Peter", 3),
            ("1 John", 5),
            ("2 John", 1),
            ("3 John", 1),
            ("Jude", 1),
            ("Revelation", 22),
        ];

        for (book, expected_chapters) in &nt_chapters {
            assert_eq!(
                bible.nt[*book].len(),
                *expected_chapters,
                "{} should have {} chapters",
                book,
                expected_chapters
            );
        }

        Ok(())
    }

    #[test]
    fn verify_specific_verse_counts() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // Genesis 1 should have 31 verses
        assert_eq!(
            bible.ot["Genesis"]["1"].len(),
            31,
            "Genesis 1 should have 31 verses"
        );

        // Psalm 119 should have 176 verses (longest chapter)
        assert_eq!(
            bible.ot["Psalms"]["119"].len(),
            176,
            "Psalm 119 should have 176 verses"
        );

        // John 3 should have 36 verses
        assert_eq!(
            bible.nt["John"]["3"].len(),
            36,
            "John 3 should have 36 verses"
        );

        // Romans 8 should have 39 verses
        assert_eq!(
            bible.nt["Romans"]["8"].len(),
            39,
            "Romans 8 should have 39 verses"
        );

        // Matthew 5 (Sermon on the Mount) should have 48 verses
        assert_eq!(
            bible.nt["Matthew"]["5"].len(),
            48,
            "Matthew 5 should have 48 verses"
        );

        // Revelation 22 (last chapter) should have 21 verses
        assert_eq!(
            bible.nt["Revelation"]["22"].len(),
            21,
            "Revelation 22 should have 21 verses"
        );

        Ok(())
    }

    #[test]
    fn verify_famous_verses() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // John 3:16 - Most famous verse
        assert_eq!(
            bible.nt["John"]["3"]["16"],
            "For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life."
        );

        // Genesis 1:1 - Opening verse
        assert_eq!(
            bible.ot["Genesis"]["1"]["1"],
            "In the beginning God created the heaven and the earth."
        );

        // Psalm 23:1 - The Lord is my shepherd
        assert_eq!(
            bible.ot["Psalms"]["23"]["1"],
            "The LORD is my shepherd; I shall not want."
        );

        // Romans 8:28
        assert_eq!(
            bible.nt["Romans"]["8"]["28"],
            "And we know that all things work together for good to them that love God, to them who are the called according to his purpose."
        );

        // Jeremiah 29:11
        assert_eq!(
            bible.ot["Jeremiah"]["29"]["11"],
            "For I know the thoughts that I think toward you, saith the LORD, thoughts of peace, and not of evil, to give you an expected end."
        );

        // Philippians 4:13
        assert_eq!(
            bible.nt["Philippians"]["4"]["13"],
            "I can do all things through Christ which strengtheneth me."
        );

        // Proverbs 3:5-6 (test verse 5)
        assert_eq!(
            bible.ot["Proverbs"]["3"]["5"],
            "Trust in the LORD with all thine heart; and lean not unto thine own understanding."
        );

        // Matthew 28:19 - Great Commission
        assert_eq!(
            bible.nt["Matthew"]["28"]["19"],
            "Go ye therefore, and teach all nations, baptizing them in the name of the Father, and of the Son, and of the Holy Ghost:"
        );

        // Isaiah 40:31
        assert_eq!(
            bible.ot["Isaiah"]["40"]["31"],
            "But they that wait upon the LORD shall renew their strength; they shall mount up with wings as eagles; they shall run, and not be weary; and they shall walk, and not faint."
        );

        // 1 Corinthians 13:4 - Love chapter
        assert_eq!(
            bible.nt["1 Corinthians"]["13"]["4"],
            "Charity suffereth long, and is kind; charity envieth not; charity vaunteth not itself, is not puffed up,"
        );

        Ok(())
    }

    #[test]
    fn verify_verse_lengths() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // Verify that verses have reasonable lengths (not empty, not too short)
        // John 11:35 is the shortest verse: "Jesus wept."
        let shortest = &bible.nt["John"]["11"]["35"];
        assert_eq!(shortest, "Jesus wept.");
        assert!(shortest.len() < 20, "Shortest verse should be very short");

        // Check that no verses are empty
        for (book_name, chapters) in bible.ot.iter() {
            for (chapter_num, verses) in chapters.iter() {
                for (verse_num, text) in verses.iter() {
                    assert!(
                        !text.is_empty(),
                        "Empty verse found in OT {} {}:{}",
                        book_name,
                        chapter_num,
                        verse_num
                    );
                    assert!(
                        text.len() >= 2,
                        "Suspiciously short verse in OT {} {}:{}: '{}'",
                        book_name,
                        chapter_num,
                        verse_num,
                        text
                    );
                }
            }
        }

        for (book_name, chapters) in bible.nt.iter() {
            for (chapter_num, verses) in chapters.iter() {
                for (verse_num, text) in verses.iter() {
                    assert!(
                        !text.is_empty(),
                        "Empty verse found in NT {} {}:{}",
                        book_name,
                        chapter_num,
                        verse_num
                    );
                    assert!(
                        text.len() >= 2,
                        "Suspiciously short verse in NT {} {}:{}: '{}'",
                        book_name,
                        chapter_num,
                        verse_num,
                        text
                    );
                }
            }
        }

        // Check some longer verses
        // Esther 8:9 is one of the longest verses
        let long_verse = &bible.ot["Esther"]["8"]["9"];
        assert!(
            long_verse.len() > 300,
            "Esther 8:9 should be a long verse (>300 chars), got {}",
            long_verse.len()
        );

        Ok(())
    }
}
