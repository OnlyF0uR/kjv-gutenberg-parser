use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Encode, Decode, Serialize, Deserialize, Clone)]
struct Verse {
    number: String,
    text: String,
}

#[derive(Encode, Decode, Serialize, Deserialize, Clone)]
struct Chapter {
    number: String,
    verses: Vec<Verse>,
}

#[derive(Encode, Decode, Serialize, Deserialize, Clone)]
struct Book {
    name: String,
    chapters: Vec<Chapter>,
}

#[derive(Encode, Decode, Serialize, Deserialize)]
struct Bible {
    ot_contents: Vec<String>, // Old Testament table of contents
    ot: Vec<Book>,            // Old Testament
    nt_contents: Vec<String>, // New Testament table of contents
    nt: Vec<Book>,            // New Testament
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
        "Ezra" => return Some(("Ezra".to_string(), true)),
        "Ecclesiastes" => return Some(("Ecclesiastes".to_string(), true)),
        _ => {} // Continue to check prefixes
    }

    // Old Testament books - checking in order with flexible matching
    // Only match if the line STARTS with the prefix (not just contains it)
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
        ("The Book of Nehemiah", "Nehemiah"),
        ("The Book of Esther", "Esther"),
        ("The Book of Job", "Job"),
        ("The Book of Psalms", "Psalms"),
        ("The Proverbs", "Proverbs"),
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
        ot_contents: Vec::new(),
        nt_contents: Vec::new(),
        ot: Vec::new(),
        nt: Vec::new(),
    };

    let mut current_book: Option<Book> = None;
    let mut is_ot = true;
    let mut current_chapter: Option<Chapter> = None;
    let mut current_verse: Option<Verse> = None;

    let mut in_bible = false;
    let mut in_content = false; // Skip table of contents
    let mut in_toc = false; // Track if we're in the table of contents
    let mut toc_is_ot = true; // Track which testament's TOC we're in
    let mut toc_complete = false; // Track when we've finished collecting TOC
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

        // Detect start of table of contents
        if !in_toc && !in_content && line.contains("The Old Testament") {
            in_toc = true;
            toc_is_ot = true;
            continue;
        }

        if in_toc && line.contains("The New Testament") {
            toc_is_ot = false;
            continue;
        }

        // Collect table of contents entries while in TOC
        if in_toc && !toc_complete {
            if let Some((book_name, _)) = is_book_line(line) {
                if toc_is_ot {
                    bible.ot_contents.push(book_name);
                } else {
                    bible.nt_contents.push(book_name);
                }

                // Check if we've collected all books (39 OT + 27 NT)
                if bible.ot_contents.len() == 39 && bible.nt_contents.len() == 27 {
                    toc_complete = true;
                    eprintln!(
                        "TOC complete: {} OT and {} NT entries",
                        bible.ot_contents.len(),
                        bible.nt_contents.len()
                    );
                }
            }
            continue;
        }

        // After TOC is complete, look for the content section marker
        if toc_complete && !in_content {
            if line.contains("The Old Testament") {
                in_content = true;
                in_toc = false;
                eprintln!("Content section starts");
                continue;
            }
            continue;
        }

        // Now we're in content - process normally
        if !in_content {
            continue;
        }

        // Check for book line first
        if let Some((book_name, is_old_testament)) = is_book_line(line) {
            // Handle Kings/Samuel edge case
            if last_line_was_book && (book_name == "1 Kings" || book_name == "2 Kings") {
                eprintln!("Skipping {} as it's part of Samuel header", book_name);
                continue;
            }

            if (book_name == "1 Kings" || book_name == "2 Kings")
                && current_book
                    .as_ref()
                    .is_some_and(|b| b.name.contains("Samuel"))
                && current_verse.is_none()
            {
                eprintln!(
                    "Preventing switch from {} to {} (no verses yet)",
                    current_book.as_ref().unwrap().name,
                    book_name
                );
                continue;
            }

            if !found_books.contains(&book_name) {
                eprintln!("Found new book: '{}' from line: '{}'", book_name, line);
                found_books.insert(book_name.clone());
            } else {
                eprintln!("Re-encountered book: '{}' from line: '{}'", book_name, line);
            }

            // Save previous verse if exists
            if let Some(verse) = current_verse.take()
                && let Some(chapter) = current_chapter.as_mut() {
                    chapter.verses.push(verse);
                }

            // Save previous chapter if exists
            if let Some(chapter) = current_chapter.take()
                && let Some(book) = current_book.as_mut() {
                    book.chapters.push(chapter);
                }

            // Save previous book if exists
            if let Some(book) = current_book.take() {
                let testament = if is_ot { &mut bible.ot } else { &mut bible.nt };
                testament.push(book);
            }

            current_book = Some(Book {
                name: book_name,
                chapters: Vec::new(),
            });
            is_ot = is_old_testament;
            last_line_was_book = true;
            continue;
        }

        last_line_was_book = false;

        // Look for verse references anywhere in the line
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut found_verse_ref = false;

        for (i, word) in words.iter().enumerate() {
            if let Some((ch, v)) = word.split_once(':')
                && ch.parse::<u32>().is_ok() && v.parse::<u32>().is_ok() {
                    // Found a verse reference!

                    // First, save the previous verse if we have one
                    if let Some(mut verse) = current_verse.take() {
                        // Append any text before this verse reference to the current verse
                        if i > 0 {
                            if !verse.text.is_empty() {
                                verse.text.push(' ');
                            }
                            verse.text.push_str(&words[..i].join(" "));
                        }

                        if let Some(chapter) = current_chapter.as_mut() {
                            chapter.verses.push(verse);
                        }
                    }

                    // Check if we need a new chapter
                    let chapter_num = ch.to_string();
                    if current_chapter
                        .as_ref()
                        .is_none_or(|c| c.number != chapter_num)
                    {
                        // Save the previous chapter if exists
                        if let Some(chapter) = current_chapter.take()
                            && let Some(book) = current_book.as_mut() {
                                book.chapters.push(chapter);
                            }
                        current_chapter = Some(Chapter {
                            number: chapter_num,
                            verses: Vec::new(),
                        });
                    }

                    // Start the new verse
                    let verse_text = if i + 1 < words.len() {
                        words[i + 1..].join(" ")
                    } else {
                        String::new()
                    };

                    current_verse = Some(Verse {
                        number: v.to_string(),
                        text: verse_text,
                    });

                    found_verse_ref = true;
                    break;
                }
        }

        // If no verse reference found, this is continuation text for current verse
        if !found_verse_ref
            && let Some(verse) = current_verse.as_mut() {
                if !verse.text.is_empty() {
                    verse.text.push(' ');
                }
                verse.text.push_str(line);
            }
    }

    // Save the last verse
    if let Some(verse) = current_verse.take()
        && let Some(chapter) = current_chapter.as_mut() {
            chapter.verses.push(verse);
        }

    // Save the last chapter
    if let Some(chapter) = current_chapter.take()
        && let Some(book) = current_book.as_mut() {
            book.chapters.push(chapter);
        }

    // Save the last book
    if let Some(book) = current_book.take() {
        let testament = if is_ot { &mut bible.ot } else { &mut bible.nt };
        testament.push(book);
    }

    eprintln!(
        "\nFinal book counts - OT: {}, NT: {}",
        bible.ot.len(),
        bible.nt.len()
    );
    eprintln!(
        "OT books: {:?}",
        bible.ot.iter().map(|b| &b.name).collect::<Vec<_>>()
    );

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

fn write_bible_to_json(bible: &Bible, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::create(path)?;
    let writer = BufWriter::new(f);
    serde_json::to_writer_pretty(writer, bible)?;
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
    println!(
        "Table of Contents - OT ({} books):",
        bible.ot_contents.len()
    );
    for (i, book) in bible.ot_contents.iter().enumerate() {
        println!("  {}. {}", i + 1, book);
    }

    println!(
        "\nTable of Contents - NT ({} books):",
        bible.nt_contents.len()
    );
    for (i, book) in bible.nt_contents.iter().enumerate() {
        println!("  {}. {}", i + 1, book);
    }

    println!("\nParsed {} OT books:", bible.ot.len());
    for (i, book) in bible.ot.iter().enumerate() {
        println!("  {}. {}", i + 1, book.name);
    }

    println!("\nParsed {} NT books:", bible.nt.len());
    for (i, book) in bible.nt.iter().enumerate() {
        println!("  {}. {}", i + 1, book.name);
    }

    write_bible_to_bin(&bible, "bible.bin")?;
    write_bible_to_json(&bible, "bible.json")?;

    println!("\nParsed KJV and saved to bible.bin successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to find a book by name
    fn find_book<'a>(testament: &'a [Book], name: &str) -> &'a Book {
        testament
            .iter()
            .find(|b| b.name == name)
            .unwrap_or_else(|| panic!("Book {} not found", name))
    }

    // Helper function to find a chapter by number
    fn find_chapter<'a>(book: &'a Book, number: &str) -> &'a Chapter {
        book.chapters
            .iter()
            .find(|c| c.number == number)
            .unwrap_or_else(|| panic!("Chapter {} not found in {}", number, book.name))
    }

    // Helper function to find a verse by number
    fn find_verse<'a>(chapter: &'a Chapter, number: &str) -> &'a Verse {
        chapter
            .verses
            .iter()
            .find(|v| v.number == number)
            .unwrap_or_else(|| panic!("Verse {} not found", number))
    }

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
        let genesis = find_book(&bible.ot, "Genesis");
        let gen_ch1 = find_chapter(genesis, "1");
        let gen_1_1 = find_verse(gen_ch1, "1");
        assert_eq!(
            gen_1_1.text,
            "In the beginning God created the heaven and the earth."
        );

        // Verify chapter counts
        assert_eq!(
            genesis.chapters.len(),
            50,
            "Genesis should have 50 chapters"
        );

        let matthew = find_book(&bible.nt, "Matthew");
        assert_eq!(
            matthew.chapters.len(),
            28,
            "Matthew should have 28 chapters"
        );

        // Matthew 3:1
        let matt_ch3 = find_chapter(matthew, "3");
        let matt_3_1 = find_verse(matt_ch3, "1");
        assert_eq!(
            matt_3_1.text,
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

        for (book_name, expected_chapters) in &ot_chapters {
            let book = find_book(&bible.ot, book_name);
            let actual_chapters = book.chapters.len();
            if actual_chapters != *expected_chapters {
                eprintln!(
                    "\n{} has {} chapters (expected {}). Chapters: {:?}",
                    book_name,
                    actual_chapters,
                    expected_chapters,
                    book.chapters.iter().map(|c| &c.number).collect::<Vec<_>>()
                );
            }
            assert_eq!(
                actual_chapters, *expected_chapters,
                "{} should have {} chapters",
                book_name, expected_chapters
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

        for (book_name, expected_chapters) in &nt_chapters {
            let book = find_book(&bible.nt, book_name);
            assert_eq!(
                book.chapters.len(),
                *expected_chapters,
                "{} should have {} chapters",
                book_name,
                expected_chapters
            );
        }

        Ok(())
    }

    #[test]
    fn verify_specific_verse_counts() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // Genesis 1 should have 31 verses
        let genesis = find_book(&bible.ot, "Genesis");
        let gen_ch1 = find_chapter(genesis, "1");
        assert_eq!(gen_ch1.verses.len(), 31, "Genesis 1 should have 31 verses");

        // Psalm 119 should have 176 verses (longest chapter)
        let psalms = find_book(&bible.ot, "Psalms");
        let ps_119 = find_chapter(psalms, "119");
        assert_eq!(ps_119.verses.len(), 176, "Psalm 119 should have 176 verses");

        // John 3 should have 36 verses
        let john = find_book(&bible.nt, "John");
        let john_ch3 = find_chapter(john, "3");
        assert_eq!(john_ch3.verses.len(), 36, "John 3 should have 36 verses");

        // Romans 8 should have 39 verses
        let romans = find_book(&bible.nt, "Romans");
        let rom_ch8 = find_chapter(romans, "8");
        assert_eq!(rom_ch8.verses.len(), 39, "Romans 8 should have 39 verses");

        // Matthew 5 (Sermon on the Mount) should have 48 verses
        let matthew = find_book(&bible.nt, "Matthew");
        let matt_ch5 = find_chapter(matthew, "5");
        assert_eq!(matt_ch5.verses.len(), 48, "Matthew 5 should have 48 verses");

        // Revelation 22 (last chapter) should have 21 verses
        let revelation = find_book(&bible.nt, "Revelation");
        let rev_ch22 = find_chapter(revelation, "22");
        assert_eq!(
            rev_ch22.verses.len(),
            21,
            "Revelation 22 should have 21 verses"
        );

        Ok(())
    }

    #[test]
    fn verify_famous_verses() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // John 3:16 - Most famous verse
        let john = find_book(&bible.nt, "John");
        let john_ch3 = find_chapter(john, "3");
        let john_3_16 = find_verse(john_ch3, "16");
        assert_eq!(
            john_3_16.text,
            "For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life."
        );

        // Genesis 1:1 - Opening verse
        let genesis = find_book(&bible.ot, "Genesis");
        let gen_ch1 = find_chapter(genesis, "1");
        let gen_1_1 = find_verse(gen_ch1, "1");
        assert_eq!(
            gen_1_1.text,
            "In the beginning God created the heaven and the earth."
        );

        // Psalm 23:1 - The Lord is my shepherd
        let psalms = find_book(&bible.ot, "Psalms");
        let ps_23 = find_chapter(psalms, "23");
        let ps_23_1 = find_verse(ps_23, "1");
        assert_eq!(ps_23_1.text, "The LORD is my shepherd; I shall not want.");

        // Romans 8:28
        let romans = find_book(&bible.nt, "Romans");
        let rom_ch8 = find_chapter(romans, "8");
        let rom_8_28 = find_verse(rom_ch8, "28");
        assert_eq!(
            rom_8_28.text,
            "And we know that all things work together for good to them that love God, to them who are the called according to his purpose."
        );

        // Jeremiah 29:11
        let jeremiah = find_book(&bible.ot, "Jeremiah");
        let jer_ch29 = find_chapter(jeremiah, "29");
        let jer_29_11 = find_verse(jer_ch29, "11");
        assert_eq!(
            jer_29_11.text,
            "For I know the thoughts that I think toward you, saith the LORD, thoughts of peace, and not of evil, to give you an expected end."
        );

        // Philippians 4:13
        let philippians = find_book(&bible.nt, "Philippians");
        let phil_ch4 = find_chapter(philippians, "4");
        let phil_4_13 = find_verse(phil_ch4, "13");
        assert_eq!(
            phil_4_13.text,
            "I can do all things through Christ which strengtheneth me."
        );

        // Proverbs 3:5-6 (test verse 5)
        let proverbs = find_book(&bible.ot, "Proverbs");
        let prov_ch3 = find_chapter(proverbs, "3");
        let prov_3_5 = find_verse(prov_ch3, "5");
        assert_eq!(
            prov_3_5.text,
            "Trust in the LORD with all thine heart; and lean not unto thine own understanding."
        );

        // Matthew 28:19 - Great Commission
        let matthew = find_book(&bible.nt, "Matthew");
        let matt_ch28 = find_chapter(matthew, "28");
        let matt_28_19 = find_verse(matt_ch28, "19");
        assert_eq!(
            matt_28_19.text,
            "Go ye therefore, and teach all nations, baptizing them in the name of the Father, and of the Son, and of the Holy Ghost:"
        );

        // Isaiah 40:31
        let isaiah = find_book(&bible.ot, "Isaiah");
        let isa_ch40 = find_chapter(isaiah, "40");
        let isa_40_31 = find_verse(isa_ch40, "31");
        assert_eq!(
            isa_40_31.text,
            "But they that wait upon the LORD shall renew their strength; they shall mount up with wings as eagles; they shall run, and not be weary; and they shall walk, and not faint."
        );

        // 1 Corinthians 13:4 - Love chapter
        let cor1 = find_book(&bible.nt, "1 Corinthians");
        let cor1_ch13 = find_chapter(cor1, "13");
        let cor1_13_4 = find_verse(cor1_ch13, "4");
        assert_eq!(
            cor1_13_4.text,
            "Charity suffereth long, and is kind; charity envieth not; charity vaunteth not itself, is not puffed up,"
        );

        Ok(())
    }

    #[test]
    fn verify_verse_lengths() -> Result<(), Box<dyn std::error::Error>> {
        let bible = get_bible();

        // Verify that verses have reasonable lengths (not empty, not too short)
        // John 11:35 is the shortest verse: "Jesus wept."
        let john = find_book(&bible.nt, "John");
        let john_ch11 = find_chapter(john, "11");
        let john_11_35 = find_verse(john_ch11, "35");
        assert_eq!(john_11_35.text, "Jesus wept.");
        assert!(
            john_11_35.text.len() < 20,
            "Shortest verse should be very short"
        );

        // Check that no verses are empty
        for book in bible.ot.iter() {
            for chapter in book.chapters.iter() {
                for verse in chapter.verses.iter() {
                    assert!(
                        !verse.text.is_empty(),
                        "Empty verse found in OT {} {}:{}",
                        book.name,
                        chapter.number,
                        verse.number
                    );
                    assert!(
                        verse.text.len() >= 2,
                        "Suspiciously short verse in OT {} {}:{}: '{}'",
                        book.name,
                        chapter.number,
                        verse.number,
                        verse.text
                    );
                }
            }
        }

        for book in bible.nt.iter() {
            for chapter in book.chapters.iter() {
                for verse in chapter.verses.iter() {
                    assert!(
                        !verse.text.is_empty(),
                        "Empty verse found in NT {} {}:{}",
                        book.name,
                        chapter.number,
                        verse.number
                    );
                    assert!(
                        verse.text.len() >= 2,
                        "Suspiciously short verse in NT {} {}:{}: '{}'",
                        book.name,
                        chapter.number,
                        verse.number,
                        verse.text
                    );
                }
            }
        }

        // Check some longer verses
        // Esther 8:9 is one of the longest verses
        let esther = find_book(&bible.ot, "Esther");
        let esther_ch8 = find_chapter(esther, "8");
        let esther_8_9 = find_verse(esther_ch8, "9");
        assert!(
            esther_8_9.text.len() > 300,
            "Esther 8:9 should be a long verse (>300 chars), got {}",
            esther_8_9.text.len()
        );

        Ok(())
    }
}
