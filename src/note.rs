use nom::{branch::alt, bytes::complete::tag, combinator::map};
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Sub},
    str::FromStr,
};

/// A note, represented as a single positive integer (called it's `value`).
/// A `value` of 0 = C0, 1 = Db0 etc.
///
/// You can create a `Note` by either parsing from a string or directly
/// inputting a number value:
///
/// ```rust
/// use minstrel::Note;
/// use std::str::FromStr;
///
/// // Directly
/// let A0 = Note::new(9);
///
/// // From a string
/// let Eb100 = Note::from_str("Eb100");
/// ```
///
/// `Note`s have a number of useful features. For example, you can easily
/// transpose one by using the addition or subtraction operators:
///
/// ```rust
/// use minstrel::Note;
///
/// let C1 = Note::new(12);
/// assert_eq!(C1 + 5, Note::new(17));
/// assert_eq!(C1 - 2, Note::new(10));
/// ```
///
/// You can also get the semitone difference between two `Note`s just by
/// subtracting them:
///
/// ```rust
/// use minstrel::Note;
///
/// let C0 = Note::new(0);
/// let E0 = Note::new(4);
///
/// // It doesn't matter which order the notes are in
/// assert_eq!(C0 - E0, 4);
/// assert_eq!(E0 - C0, 4);
/// ```
///
/// Finally, you can call `into_iter` on a `Note` to iterate over it:
///
/// ```rust
/// use minstrel::Note;
///
/// // Prints the chromatic scale
/// for note in Note::new(0).into_iter().take(12) {
///    println!("{}", note);
/// }
/// ```
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Note {
    pub value: usize,
}

impl Note {
    /// Creates a new `Note` with the given `value`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minstrel::Note;
    ///
    /// let C0 = Note::new(0);
    /// let F4 = Note::new(53);
    /// let Ab5 = Note::new(68);
    /// ```
    pub fn new(value: usize) -> Self {
        Note { value }
    }

    /// Returns a new `Note` where the inner `value` holds no octave information
    /// i.e. it is constrained between 0 and 11.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minstrel::Note;
    ///
    /// let F = Note::new(53).disregard_octave();
    ///
    /// let Ab5 = Note::new(68);
    /// let Ab = Ab5.disregard_octave();
    /// ```
    pub fn disregard_octave(self) -> Self {
        Self {
            value: self.value % 12,
        }
    }
}

impl FromStr for Note {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, name) = alt((
            map(tag("C"), |_| 0),
            map(tag("Db"), |_| 1),
            map(tag("D"), |_| 2),
            map(tag("Eb"), |_| 3),
            map(tag("E"), |_| 4),
            map(tag("F"), |_| 5),
            map(tag("Gb"), |_| 6),
            map(tag("G"), |_| 7),
            map(tag("Ab"), |_| 8),
            map(tag("A"), |_| 9),
            map(tag("Bb"), |_| 10),
            map(tag("B"), |_| 11),
        ))(s)
        .map_err(|_: nom::Err<(&str, nom::error::ErrorKind)>| {
            anyhow::anyhow!("failed to parse note name")
        })?;

        // Gives an octave value of 0 if none was supplied
        let octave = if s.is_empty() { 0 } else { usize::from_str(s)? };

        Ok(Self::new(name + octave * 12))
    }
}

#[cfg(test)]
#[test]
fn parsing() {
    assert_eq!(Note::from_str("C0").unwrap(), Note::new(0));
    assert_eq!(Note::from_str("Db3").unwrap(), Note::new(37));
    assert_eq!(Note::from_str("Bb10").unwrap(), Note::new(130));
    assert_eq!(Note::from_str("Ab").unwrap(), Note::new(8));

    assert!(Note::from_str("Cb2").is_err()); // Invalid note name
    assert!(Note::from_str("Gb-2").is_err()); // Invalid octave number
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.value % 12 {
            0 => "C",
            1 => "Db",
            2 => "D",
            3 => "Eb",
            4 => "E",
            5 => "F",
            6 => "Gb",
            7 => "G",
            8 => "Ab",
            9 => "A",
            10 => "Bb",
            11 => "B",
            _ => unreachable!(),
        };

        if f.alternate() {
            let octave = self.value / 12;
            write!(f, "{}{}", name, octave)
        } else {
            write!(f, "{}", name)
        }
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn normal() {
        assert_eq!(Note::new(0).to_string(), "C");
        assert_eq!(Note::new(37).to_string(), "Db");
        assert_eq!(Note::new(76).to_string(), "E");
    }

    #[test]
    fn alternate() {
        assert_eq!(format!("{:#}", Note::new(0)), "C0");
        assert_eq!(format!("{:#}", Note::new(37)), "Db3");
        assert_eq!(format!("{:#}", Note::new(76)), "E6");
    }
}

impl Add<usize> for Note {
    type Output = Self;

    fn add(self, semitones: usize) -> Self::Output {
        Self {
            value: self.value + semitones,
        }
    }
}

impl Sub<usize> for Note {
    type Output = Self;

    fn sub(self, semitones: usize) -> Self::Output {
        Self {
            value: self.value - semitones,
        }
    }
}

#[cfg(test)]
#[test]
fn transposition() {
    assert_eq!(Note::new(10) + 5, Note::new(15));
    assert_eq!(Note::new(42) + 12, Note::new(54));
    assert_eq!(Note::new(10) - 5, Note::new(5));
    assert_eq!(Note::new(42) - 12, Note::new(30));
}

impl Sub for Note {
    type Output = usize;

    // Outputs the semitone difference between the two note values
    fn sub(self, other: Self) -> Self::Output {
        match self.value.cmp(&other.value) {
            Ordering::Greater => self.value - other.value,
            Ordering::Less => other.value - self.value,
            Ordering::Equal => 0,
        }
    }
}

#[cfg(test)]
#[test]
fn interval_calculation() {
    assert_eq!(Note::new(10) - Note::new(5), 5);
    assert_eq!(Note::new(21) - Note::new(27), 6);
    assert_eq!(Note::new(37) - Note::new(37), 0);
}

impl IntoIterator for Note {
    type Item = Self;
    type IntoIter = NoteIter;

    fn into_iter(self) -> Self::IntoIter {
        NoteIter {
            note: self,
            first: true,
        }
    }
}

/// An iterator over a `Note`.
pub struct NoteIter {
    note: Note,
    first: bool,
}

impl Iterator for NoteIter {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        // Returns the original note if this was the first iteration
        if self.first {
            self.first = false;
            Some(self.note)
        } else {
            self.note.value += 1;
            Some(self.note)
        }
    }
}
