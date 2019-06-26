//
// Parse the formal preferences CSV file
// Example file: http://results.aec.gov.au/20499/Website/External/aec-senate-formalpreferences-20499-NT.zip
//

extern crate csv;
extern crate flate2;

use defs::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::iter;

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


struct PrefParser {
    ticket_forms: Vec<Vec<CandidateIndex>>,
    tickets: usize,
    atl: Vec<ATLPref>,
    btl: Vec<BTLPref>,
}

impl PrefParser {
    fn new(tickets: &[Vec<CandidateIndex>], candidates: usize) -> PrefParser {
        PrefParser {
            ticket_forms: tickets.to_vec(),
            tickets: tickets.len(),
            atl: Vec::with_capacity(tickets.len()),
            btl: Vec::with_capacity(candidates),
        }
    }

    fn clear(self: &mut Self) {
        self.atl.clear();
        self.btl.clear();
    }

    fn sort(self: &mut Self) {
        self.atl.sort();
        self.btl.sort();
    }

    fn parse(self: &mut Self, pref: &str, mut form_buf: &mut ResolvedPrefs) {
        self.clear();
        self.parse_line(pref);
        self.sort();
        self.expand(&mut form_buf);
    }

    // note: this function could be a lot neater, or just use the csv library, but
    // it's performance critical and so is hand optimised. we can assume that we're
    // plain ASCII, that the field values are either empty or are a smallish integer
    fn parse_line(self: &mut Self, prefs: &str) {
        let mut field = 0;
        let mut from = 0;

        let mut it = prefs.bytes();
        let mut upto: usize = 0;
        loop {
            let n = it.next();
            let mut eol = false;
            let term = match n {
                Some(c) => c == b',',
                None => {
                    eol = true;
                    true
                }
            };
            if term {
                if upto - from > 0 {
                    let pref = pref_to_u8(&prefs[from..upto]);
                    if field < self.tickets {
                        self.atl.push((GroupPreference(pref), GroupIndex(field as u8)));
                    } else {
                        self.btl.push((
                            CandidatePreference(pref),
                            CandidateIndex((field - self.tickets) as u8),
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
    }

    fn expand_btl(self: &Self, form_buf: &mut ResolvedPrefs) {
        let left = self.btl.iter();
        let right = self.btl.iter().map(Some).skip(1).chain(iter::once(None));
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

    fn expand_atl(self: &Self, form_buf: &mut ResolvedPrefs) {
        let left = self.atl.iter();
        let right = self.atl.iter().map(Some).skip(1).chain(iter::once(None));
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
            form_buf.extend(&self.ticket_forms[(pref.1).0 as usize]);
        }
    }

    fn expand(
        self: &Self,
        mut form_buf: &mut ResolvedPrefs,
    ) {
        // if we have at least six BTL prefrences, we have a valid form
        self.expand_btl(&mut form_buf);
        if form_buf.len() < 6 {
            // we don't have a valid BTL form, validate and expand above-the-line
            // preferences
            form_buf.clear();
            self.expand_atl(&mut form_buf);
        }
    }

}

fn process_fd(
    fd: impl std::io::Read,
    tickets: &[Vec<CandidateIndex>],
    candidates: usize,
) -> Vec<BallotState> {
    let rdr = BufReader::new(fd);
    let mut form_counter: HashMap<ResolvedPrefs, u32> = HashMap::new();
    let mut parser = PrefParser::new(&tickets, candidates);

    for r in rdr.lines().skip(2) {
        let line = r.unwrap();
        let pref: &str = &line[(line.find('\"').unwrap() + 1)..line.len() - 1];
        let mut form_buf: ResolvedPrefs = Vec::with_capacity(candidates);

        parser.parse(pref, &mut form_buf);
        assert!(!form_buf.is_empty());

        let counter = form_counter.entry(form_buf).or_insert(0);
        *counter += 1;
    }

    let v: Vec<BallotState> = form_counter
        .drain()
        .map(|(form, count)| BallotState {
            form,
            count,
            active_preference: 0,
        })
        .collect();
    v
}

pub fn read_file(
    filename: &str,
    tickets: &[Vec<CandidateIndex>],
    candidates: usize,
) -> Vec<BallotState> {
    let f = File::open(filename).unwrap();
    let gf = flate2::read::GzDecoder::new(f);
    process_fd(gf, tickets, candidates)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_prefstring(
        tickets: usize,
        line: &str,
        atl_expected: &Vec<ATLPref>,
        btl_expected: &Vec<BTLPref>,
    ) {
        let mut parser = PrefParser::new(tickets, 0);
        parser.parse_line(&line, tickets);
        assert!(*atl_expected == parser.atl);
        assert!(*btl_expected == parser.btl);
    }

    #[test]
    fn prefstring_atl_only() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(2), GroupIndex(0)),
            (GroupPreference(3), GroupIndex(2)),
        ]
        .to_vec();
        let btl: Vec<BTLPref> = [].to_vec();

        parse_prefstring(3, &String::from("2,1,3,,,"), &atl, &btl);
    }

    #[test]
    fn prefstring_atl_and_btl() {
        let atl: Vec<ATLPref> = [(GroupPreference(1), GroupIndex(2))].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(12)),
            (CandidatePreference(2), CandidateIndex(11)),
        ]
        .to_vec();

        parse_prefstring(3, &String::from(",,1,,,,,,,,,,,,2,1"), &atl, &btl);
    }

    #[test]
    fn prefstring_full_line() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(0)),
            (GroupPreference(2), GroupIndex(1)),
            (GroupPreference(3), GroupIndex(2)),
        ]
        .to_vec();
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
        ]
        .to_vec();

        parse_prefstring(
            3,
            &String::from("1,2,3,1,2,3,4,5,6,7,8,9,10,11,12"),
            &atl,
            &btl,
        );
    }

    #[test]
    fn prefstring_prefstring_btl_only() {
        let atl: Vec<ATLPref> = [].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(2), CandidateIndex(0)),
            (CandidatePreference(3), CandidateIndex(2)),
        ]
        .to_vec();

        parse_prefstring(3, &String::from(",,,2,1,3"), &atl, &btl);
    }

    #[test]
    fn expandatl_simple() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(2), GroupIndex(0)),
            (GroupPreference(3), GroupIndex(2)),
        ]
        .to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ]
        .to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(
            form_buf
                == [
                    CandidateIndex(2),
                    CandidateIndex(0),
                    CandidateIndex(1),
                    CandidateIndex(3),
                    CandidateIndex(4),
                    CandidateIndex(5),
                ]
                .to_vec()
        );
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
        ]
        .to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ]
        .to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(form_buf == [CandidateIndex(2),].to_vec());
    }

    #[test]
    fn expandatl_prefdupe() {
        let atl: Vec<ATLPref> = [
            (GroupPreference(1), GroupIndex(1)),
            (GroupPreference(2), GroupIndex(0)),
            (GroupPreference(2), GroupIndex(2)),
        ]
        .to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ]
        .to_vec();

        let mut form_buf = Vec::new();
        expand_atl(&atl, &mut form_buf, &tickets);
        assert!(form_buf == [CandidateIndex(2),].to_vec());
    }

    #[test]
    fn expandbtl_simple() {
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(1)),
            (CandidatePreference(2), CandidateIndex(0)),
            (CandidatePreference(3), CandidateIndex(2)),
        ]
        .to_vec();

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
        ]
        .to_vec();

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
        ]
        .to_vec();

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
        ]
        .to_vec();

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
        ]
        .to_vec();

        let mut form_buf = Vec::new();
        expand_btl(&btl, &mut form_buf);
        assert!(form_buf == [CandidateIndex(1), CandidateIndex(3), CandidateIndex(0)]);
    }

    #[test]
    fn expand_btlonly() {
        let atl: Vec<ATLPref> = [].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(0)),
            (CandidatePreference(2), CandidateIndex(1)),
            (CandidatePreference(3), CandidateIndex(2)),
            (CandidatePreference(4), CandidateIndex(3)),
            (CandidatePreference(5), CandidateIndex(4)),
            (CandidatePreference(6), CandidateIndex(5)),
        ]
        .to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> = &[].to_vec();
        let mut form_buf = Vec::new();
        expand(&PrefParser { atl, btl }, &mut form_buf, &tickets);
        assert!(
            form_buf
                == [
                    CandidateIndex(0),
                    CandidateIndex(1),
                    CandidateIndex(2),
                    CandidateIndex(3),
                    CandidateIndex(4),
                    CandidateIndex(5)
                ]
        );
    }

    #[test]
    fn expand_atlonly() {
        let atl: Vec<ATLPref> = [(GroupPreference(1), GroupIndex(0))].to_vec();
        let btl: Vec<BTLPref> = [].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> =
            &[[CandidateIndex(0), CandidateIndex(1)].to_vec()].to_vec();
        let mut form_buf = Vec::new();
        expand(&PrefParser { atl, btl }, &mut form_buf, &tickets);
        assert!(form_buf == [CandidateIndex(0), CandidateIndex(1)]);
    }

    #[test]
    fn expand_btl_beats_atl() {
        let atl: Vec<ATLPref> = [(GroupPreference(1), GroupIndex(0))].to_vec();
        let btl: Vec<BTLPref> = [
            (CandidatePreference(1), CandidateIndex(0)),
            (CandidatePreference(2), CandidateIndex(1)),
            (CandidatePreference(3), CandidateIndex(2)),
            (CandidatePreference(4), CandidateIndex(3)),
            (CandidatePreference(5), CandidateIndex(4)),
            (CandidatePreference(6), CandidateIndex(5)),
        ]
        .to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> =
            &[[CandidateIndex(0), CandidateIndex(1)].to_vec()].to_vec();
        let mut form_buf = Vec::new();
        expand(&PrefParser { atl, btl }, &mut form_buf, &tickets);
        assert!(
            form_buf
                == [
                    CandidateIndex(0),
                    CandidateIndex(1),
                    CandidateIndex(2),
                    CandidateIndex(3),
                    CandidateIndex(4),
                    CandidateIndex(5)
                ]
        );
    }

    #[test]
    fn expand_atl_rescues_btl() {
        let atl: Vec<ATLPref> = [(GroupPreference(1), GroupIndex(0))].to_vec();
        let btl: Vec<BTLPref> = [(CandidatePreference(1), CandidateIndex(0))].to_vec();
        let tickets: &Vec<Vec<CandidateIndex>> =
            &[[CandidateIndex(0), CandidateIndex(1)].to_vec()].to_vec();
        let mut form_buf = Vec::new();
        expand(&PrefParser { atl, btl }, &mut form_buf, &tickets);
        assert!(form_buf == [CandidateIndex(0), CandidateIndex(1)]);
    }

    fn stringify_ballotstates(ballot_states: &[BallotState]) -> String {
        let mut stringed: Vec<String> = ballot_states.iter().map(|x| format!("{:?}", x)).collect();
        stringed.sort();
        format!("{:?}", stringed)
    }

    #[test]
    fn parse_aec_csv() {
        let csv_data = r##"ElectorateNm,VoteCollectionPointNm,VoteCollectionPointId,BatchNo,PaperNo,Preferences
------------,---------------------,---------------------,-------,-------,-----------
Narnia,Cupboard,1,1,1,"1,2,3,1,2,3,4,5,6"
Narnia,Cupboard,1,1,1,"1,2,3,1,2,3,4,5,6"
Middle Earth,Rohan,42,43,1,"1,,,,,,,,"
"##;
        let fd = BufReader::new(csv_data.as_bytes());
        let tickets: &Vec<Vec<CandidateIndex>> = &[
            [CandidateIndex(0), CandidateIndex(1)].to_vec(),
            [CandidateIndex(2)].to_vec(),
            [CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)].to_vec(),
        ]
        .to_vec();
        let res = process_fd(fd, tickets, 6);
        assert!(stringify_ballotstates(&res) == r##"["BallotState { form: [CandidateIndex(0), CandidateIndex(1), CandidateIndex(2), CandidateIndex(3), CandidateIndex(4), CandidateIndex(5)], count: 2, active_preference: 0 }", "BallotState { form: [CandidateIndex(0), CandidateIndex(1)], count: 1, active_preference: 0 }"]"##);
    }
}
