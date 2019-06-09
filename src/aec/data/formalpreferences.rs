//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;
extern crate flate2;

use std::fs::File;
use std::collections::HashMap;
use std::io::BufReader;
use std::io::BufRead;
use std::iter;
use defs::*;

// a voter's numerical preference for a candidate
// if valid, it ranges from 1..N where N is the number of candidates
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct CandidatePreference(pub u8);

// a voter's numerical preference for a group
// if valid, it ranges from 1..N where N is the number of groups
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct GroupPreference(pub u8);

fn pref_to_u8(pref: &str) -> u8 {
    if pref == "*" || pref == "/" {
        1
    } else {
        pref.parse::<u8>().unwrap()
    }
}

type ATLPref = (GroupPreference, GroupIndex);
type BTLPref = (CandidatePreference, CandidateIndex);
type ResolvedPrefs = Vec<CandidateIndex>;

// note: this function could be a lot neater, or just use the csv library, but
// it's performance critical and so is hand optimised. we can assume that we're
// plain ASCII, that the field values are either empty or are a smallish integer
fn parse_line(prefs: &str, atl_buf: &mut Vec<ATLPref>, btl_buf: &mut Vec<BTLPref>, tickets: usize) {
    let mut field = 0;
    let mut from = 0;

    let mut it = prefs.bytes();
    let mut upto: usize = 0;
    loop {
        let n = it.next();
        let mut eol = false;
        let term = match n {
            Some(c) => {
                c == b','
            },
            None => {
                eol = true;
                true
            }
        };
        if term {
            if upto - from > 0 {
                let pref = pref_to_u8(&prefs[from..upto]);
                if field < tickets {
                    atl_buf.push((
                        GroupPreference(pref),
                        GroupIndex(field as u8)));
                } else {
                    btl_buf.push((
                        CandidatePreference(pref),
                        CandidateIndex((field - tickets) as u8),
                    ));
                }
            }
            field += 1;
            from = upto + 1;
        }
        if eol {
            break;
        }
        upto += 1;
    }

    atl_buf.sort();
    btl_buf.sort();
}

fn expand_btl(btl_buf: &[BTLPref], form_buf: &mut ResolvedPrefs) {
    let left = btl_buf.iter();
    let right = btl_buf.iter().map(Some).skip(1).chain(iter::once(None));
    let combo = left.zip(right).enumerate();

    // Validate below-the-line preferences. If these are valid, they take
    // precedence over any above-the-line preferences.
    for (idx, (pref, next_pref)) in combo {
        if pref.0 != CandidatePreference((idx + 1) as u8) {
            break;
        }
        // look ahead: we can't have double preferences
        if let Some(&next) = next_pref {
            if pref.0 == next.0 {
                break;
            }
        }
        form_buf.push(pref.1);
    }
}

fn expand_atl(atl_buf: &[ATLPref], form_buf: &mut ResolvedPrefs, tickets: &[Vec<CandidateIndex>]) {
    let left = atl_buf.iter();
    let right = atl_buf.iter().map(Some).skip(1).chain(iter::once(None));
    let combo = left.zip(right).enumerate();

    for (idx, (pref, next_pref)) in combo {
        if pref.0 != GroupPreference((idx + 1) as u8) {
            break;
        }
        // look ahead: we can't have double preferences
        if let Some(&next) = next_pref {
            if pref.0 == next.0 {
                break;
            }
        }
        form_buf.extend(&tickets[(pref.1).0 as usize]);
    }
}

fn expand(atl_buf: &[ATLPref], btl_buf: &Vec<BTLPref>, mut form_buf: &mut ResolvedPrefs, tickets: &[Vec<CandidateIndex>]) {
    // if we have at least six BTL prefrences, we have a valid form
    expand_btl(&btl_buf, &mut form_buf);
    if form_buf.len() < 6 {
        // we don't have a valid BTL form, validate and expand above-the-line
        // preferences
        form_buf.clear();
        expand_atl(&atl_buf, &mut form_buf, &tickets);
    }
}

pub fn read_file(filename: &str, tickets: &[Vec<CandidateIndex>], candidates: usize) -> Vec<BallotState> {
    let f = File::open(filename).unwrap();
    let gf = flate2::read::GzDecoder::new(f);
    let rdr = BufReader::new(gf);
    let mut form_counter: HashMap<ResolvedPrefs, u32> = HashMap::new();

    let mut atl_buf: Vec<ATLPref> = Vec::with_capacity(tickets.len());
    let mut btl_buf: Vec<BTLPref> = Vec::with_capacity(candidates);

    for r in rdr.lines().skip(2) {
        atl_buf.clear();
        btl_buf.clear();

        let line = r.unwrap();
        let pref: &str = &line[(line.find('\"').unwrap() + 1)..line.len() - 1];

        let mut form_buf: ResolvedPrefs = Vec::with_capacity(candidates);
        parse_line(pref, &mut atl_buf, &mut btl_buf, tickets.len());
        expand(&atl_buf, &btl_buf, &mut form_buf, tickets);
        assert!(!form_buf.is_empty());

        let counter = form_counter.entry(form_buf).or_insert(0);
        *counter += 1;
    }

    let v: Vec<BallotState> = form_counter.drain()
        .map(|(form, count)| BallotState {
            form,
            count,
            active_preference: 0,
        })
        .collect();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_prefstring(tickets: usize, line: &str, atl_expected: &Vec<ATLPref>, btl_expected: &Vec<BTLPref>) {
        let mut atl_buf: Vec<ATLPref> = Vec::new();
        let mut btl_buf: Vec<BTLPref> = Vec::new();

        parse_line(&line, &mut atl_buf, &mut btl_buf, tickets);
        assert!(*atl_expected == atl_buf);
        assert!(*btl_expected == btl_buf);
    }

    #[test]
    fn prefstring_atl_only() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(2), GroupIndex(0)),
            (GroupPreference(3), GroupIndex(2)),
        ].to_vec();
        let btl: Vec<BTLPref> = [].to_vec();

        parse_prefstring(3, &String::from("2,1,3,,,"), &atl, &btl);
    }

    #[test]
    fn prefstring_atl_and_btl() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(2)),
        ].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(12)),
            (CandidatePreference(2), CandidateIndex(11)),
        ].to_vec();

        parse_prefstring(3, &String::from(",,1,,,,,,,,,,,,2,1"), &atl, &btl);
    }

    #[test]
    fn prefstring_full_line() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(0)),
            (GroupPreference(2), GroupIndex(1)),
            (GroupPreference(3), GroupIndex(2)),
        ].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(0)),
            (CandidatePreference(2), CandidateIndex(1)),
            (CandidatePreference(3), CandidateIndex(2)),
            (CandidatePreference(4), CandidateIndex(3)),
            (CandidatePreference(5), CandidateIndex(4)),
            (CandidatePreference(6), CandidateIndex(5)),
            (CandidatePreference(7), CandidateIndex(6)),
            (CandidatePreference(8), CandidateIndex(7)),
            (CandidatePreference(9), CandidateIndex(8)),
            (CandidatePreference(10), CandidateIndex(9)),
            (CandidatePreference(11), CandidateIndex(10)),
            (CandidatePreference(12), CandidateIndex(11)),
        ].to_vec();

        parse_prefstring(3, &String::from("1,2,3,1,2,3,4,5,6,7,8,9,10,11,12"), &atl, &btl);
    }

    #[test]
    fn prefstring_prefstring_btl_only() {
        let atl: Vec<ATLPref> = [].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(2), CandidateIndex(0)),
            (CandidatePreference(3), CandidateIndex(2)),
        ].to_vec();

        parse_prefstring(3, &String::from(",,,2,1,3"), &atl, &btl);
    }

    #[test]
    fn expandatl_simple() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(2), GroupIndex(0)),
            (GroupPreference(3), GroupIndex(2)),
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(2),
            CandidateIndex(0), CandidateIndex(1),
            CandidateIndex(3), CandidateIndex(4), CandidateIndex(5),
        ].to_vec());
    }

    #[test]
    fn expandatl_empty() {
        let atl: Vec<ATLPref> = [].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[].to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(form_buf == [].to_vec());
    }

    #[test]
    fn expandatl_skippref() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(3), GroupIndex(0)),
            (GroupPreference(4), GroupIndex(2)),
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(2),
        ].to_vec());
    }

    #[test]
    fn expandatl_prefdupe() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(2), GroupIndex(0)),
            (GroupPreference(2), GroupIndex(2)),
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(2),
        ].to_vec());
    }

    #[test]
    fn expandbtl_simple() {
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(2), CandidateIndex(0)),
            (CandidatePreference(3), CandidateIndex(2)),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf == [CandidateIndex(1), CandidateIndex(0), CandidateIndex(2)].to_vec());
    }

    #[test]
    fn expandbtl_empty() {
        let btl: Vec<BTLPref> = [].to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf.is_empty());
    }

    #[test]
    fn expandbtl_skippref() {
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(3), CandidateIndex(0)),
            (CandidatePreference(4), CandidateIndex(2)),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf == [CandidateIndex(1)]);
    }

    #[test]
    fn expandbtl_prefdupe() {
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(2), CandidateIndex(3)),
            (CandidatePreference(3), CandidateIndex(0)),
            (CandidatePreference(3), CandidateIndex(5)),
            (CandidatePreference(4), CandidateIndex(2)),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf == [CandidateIndex(1), CandidateIndex(3)]);
    }

    #[test]
    fn expandbtl_prefdupe_first() {
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(1), CandidateIndex(3)),
            (CandidatePreference(2), CandidateIndex(0)),
            (CandidatePreference(3), CandidateIndex(5)),
            (CandidatePreference(4), CandidateIndex(2)),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf == []);
    }

 
    #[test]
    fn expandbtl_prefdupe_last() {
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(2), CandidateIndex(3)),
            (CandidatePreference(3), CandidateIndex(0)),
            (CandidatePreference(4), CandidateIndex(5)),
            (CandidatePreference(4), CandidateIndex(2)),
        ].to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf == [CandidateIndex(1), CandidateIndex(3), CandidateIndex(0)]);
    }

    #[test]
    fn expand_btlonly() {
        let atl: Vec<ATLPref> = [
        ].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(0)),
            (CandidatePreference(2), CandidateIndex(1)),
            (CandidatePreference(3), CandidateIndex(2)),
            (CandidatePreference(4), CandidateIndex(3)),
            (CandidatePreference(5), CandidateIndex(4)),
            (CandidatePreference(6), CandidateIndex(5)),
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[].to_vec();
        let mut form_buf = Vec::new();
        expand(&atl, &btl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(0), CandidateIndex(1),
            CandidateIndex(2), CandidateIndex(3),
            CandidateIndex(4), CandidateIndex(5)]);
    }

    #[test]
    fn expand_atlonly() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(0)),
        ].to_vec();
        let btl: Vec<BTLPref> = [
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[[CandidateIndex(0), CandidateIndex(1)].to_vec()].to_vec();
        let mut form_buf = Vec::new();
        expand(&atl, &btl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(0), CandidateIndex(1)]);
    }

    #[test]
    fn expand_btl_beats_atl() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(0)),
        ].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(0)),
            (CandidatePreference(2), CandidateIndex(1)),
            (CandidatePreference(3), CandidateIndex(2)),
            (CandidatePreference(4), CandidateIndex(3)),
            (CandidatePreference(5), CandidateIndex(4)),
            (CandidatePreference(6), CandidateIndex(5)),
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[[CandidateIndex(0), CandidateIndex(1)].to_vec()].to_vec();
        let mut form_buf = Vec::new();
        expand(&atl, &btl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(0), CandidateIndex(1),
            CandidateIndex(2), CandidateIndex(3),
            CandidateIndex(4), CandidateIndex(5)]);
    }

    #[test]
    fn expand_atl_rescues_btl() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(0)),
        ].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(0)),
        ].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[[CandidateIndex(0), CandidateIndex(1)].to_vec()].to_vec();
        let mut form_buf = Vec::new();
        expand(&atl, &btl, &mut form_buf, &tickets);
        assert!(form_buf == [
            CandidateIndex(0), CandidateIndex(1)]);
    }
}
